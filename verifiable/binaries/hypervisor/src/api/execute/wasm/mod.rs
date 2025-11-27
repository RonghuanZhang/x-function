use aes_gcm_siv::aead::Aead;
use anyhow::{anyhow, Context};
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;
use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store,
};
use wasmtime_wasi::{
    p2::{bindings::Command, pipe::MemoryOutputPipe},
    ResourceTable, WasiCtx, WasiCtxView, WasiView,
};

use crate::{
    error::HypervisorError,
    types::HypervisorState,
    utils::{
        self, commitment, crypto,
        x402::{self, X402_FIXED_PRICE_USDC},
    },
};

pub(crate) fn api_register(router: Router<HypervisorState>) -> Router<HypervisorState> {
    router.route("/test/execute/wasm", post(execute_wasm))
}

pub(crate) fn api_x402_register(
    router: Router<HypervisorState>,
    state: HypervisorState,
) -> Router<HypervisorState> {
    let x402_router = Router::new()
        .route("/test/wasm", post(execute_wasm))
        .route("/verifiable/wasm", post(verifiable_execute_wasm))
        .layer(x402::create_x402_middleware(X402_FIXED_PRICE_USDC))
        .with_state(state);

    router.nest("/x402_execute", x402_router)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifiableExecutionResponse {
    pub session_id: Uuid,
    pub encrypted_result: String,
    pub result_nonce: String,
    pub result_commitment: String,
    pub result_quote: String,
}

async fn verifiable_execute_wasm(
    state: State<HypervisorState>,
    req: Json<ExecutionRequest>,
) -> Result<Json<VerifiableExecutionResponse>, HypervisorError> {
    let Json(resp) = execute_wasm(state, req).await?;
    let commitment: [u8; 32] =
        const_hex::decode_to_array(&resp.result_commitment).expect("impossible");

    let quote = attest::get_quote(utils::attest::generate_raw_report_from_hash(commitment))
        .context("get execute result quote")
        .context(StatusCode::INTERNAL_SERVER_ERROR)?;

    let verifiable_resp = VerifiableExecutionResponse {
        session_id: resp.session_id,
        result_nonce: resp.result_nonce,
        encrypted_result: resp.encrypted_result,
        result_commitment: resp.result_commitment,
        result_quote: const_hex::encode(quote.to_bytes()),
    };

    Ok(Json(verifiable_resp))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub encrypted_wasm: String,
    #[serde(default = "Vec::new")]
    pub encrypted_arguments: Vec<String>,
    pub public_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionResponse {
    pub session_id: Uuid,
    pub encrypted_result: String,
    pub result_nonce: String,
    pub result_commitment: String,
}

#[tracing::instrument(skip(state, req), err)]
async fn execute_wasm(
    State(state): State<HypervisorState>,
    Json(req): Json<ExecutionRequest>,
) -> Result<Json<ExecutionResponse>, HypervisorError> {
    // Validate request
    validate_execution_request(&req)?;

    let start_time = std::time::Instant::now();

    let user_pk = crypto::pk_from_hex(&req.public_key)
        .context("decode request pubkey")
        .context(StatusCode::BAD_REQUEST)?;

    let (session_sk, session_id) = state
        .get_session_keypair(&user_pk)
        .ok_or(anyhow!("session not found"))
        .context(StatusCode::UNAUTHORIZED)?;

    let cipher = crypto::create_encrypt_key(&session_sk, &user_pk, session_id)
        .context("create encrypt key")
        .context(StatusCode::INTERNAL_SERVER_ERROR)?;

    let msg_nonce = crypto::derive_msg_nonce(session_id);

    let decrypted_wasm = {
        let encrypted_bytes = const_hex::decode(&req.encrypted_wasm)
            .context("invalid wasm binary hex")
            .context(StatusCode::BAD_REQUEST)?;

        cipher
            .decrypt(&msg_nonce, encrypted_bytes.as_slice())
            .map_err(|e| anyhow!(e.to_string()))
            .context("decrypt wasm binary")
            .context(StatusCode::BAD_REQUEST)?
    };

    let decrypted_arguments = { req.encrypted_arguments.iter() }
        .map(|a| {
            let bytes = const_hex::decode(a).context("decode argument hex")?;

            let decrypted = cipher
                .decrypt(&msg_nonce, bytes.as_ref())
                .map_err(|e| anyhow!(e.to_string()))
                .context("decrypt argument")?;

            let a = String::from_utf8(decrypted).context("argument isn't string")?;

            Ok(a)
        })
        .collect::<Result<Vec<String>, anyhow::Error>>()
        .context(StatusCode::BAD_REQUEST)?;

    info!(
        session_id = %session_id,
        public_key = req.public_key,
        "processing WASM execution request"
    );

    let mut config = Config::new();
    config.async_support(true);

    let engine = Engine::new(&config).unwrap();
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;

    let stdout = MemoryOutputPipe::new(4096);
    let wasi = WasiCtx::builder()
        .arg("wasm")
        .args(&decrypted_arguments)
        .stdout(stdout.clone())
        .build();

    let state = ComponentRunStates {
        wasi_ctx: wasi,
        resource_table: ResourceTable::new(),
    };
    let mut store = Store::new(&engine, state);

    let component = Component::from_binary(&engine, &decrypted_wasm)?;

    let command = Command::instantiate_async(&mut store, &component, &linker).await?;

    let app_output = match command.wasi_cli_run().call_run(&mut store).await {
        Err(e) => {
            return Err(e
                .context("execute wasm")
                .context(StatusCode::BAD_REQUEST)
                .into());
        }
        Ok(Ok(_)) => stdout.contents(),
        Ok(Err(_)) => {
            return Err(anyhow!("unexpected app exited")
                .context(StatusCode::BAD_REQUEST)
                .into())
        }
    };

    info!(
        session_id = %session_id,
        public_key = req.public_key,
        execution_time_ms = start_time.elapsed().as_millis(),
        status = "success",
        msg = "WASM execution completed successfully"
    );

    let output_nonce = crypto::derive_msg_nonce(&app_output);
    let encrypted_result = {
        let encrypted = cipher
            .encrypt(&output_nonce, app_output.as_ref())
            .map_err(|e| anyhow!(e.to_string()))?;

        const_hex::encode(encrypted)
    };

    let result_commitment = commitment::build_result_commitment(
        &user_pk,
        session_sk.verifying_key(),
        session_id,
        &req.encrypted_wasm,
        &req.encrypted_arguments,
        output_nonce,
        &encrypted_result,
    );

    let resp = ExecutionResponse {
        session_id,
        result_nonce: const_hex::encode(output_nonce),
        encrypted_result,
        result_commitment: const_hex::encode(result_commitment),
    };

    Ok(axum::Json(resp))
}

/// Validate execution request
fn validate_execution_request(request: &ExecutionRequest) -> Result<(), HypervisorError> {
    let validate = || -> anyhow::Result<()> {
        // Validate encrypted_wasm
        anyhow::ensure!(
            !request.encrypted_wasm.trim().is_empty(),
            "encrypted_wasm cannot be empty"
        );

        // Validate public_key
        anyhow::ensure!(
            !(request.public_key.trim().is_empty()),
            "public key cannot be empty"
        );

        Ok(())
    };

    validate().context(StatusCode::BAD_REQUEST)?;

    Ok(())
}

struct ComponentRunStates {
    wasi_ctx: WasiCtx,
    resource_table: ResourceTable,
}

impl WasiView for ComponentRunStates {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi_ctx,
            table: &mut self.resource_table,
        }
    }
}

#[cfg(test)]
mod tests {
    use aes_gcm_siv::{aead::Aead, Nonce};

    use crate::utils::crypto;
    use crate::{api::RouterRegister, types::SessionKeyPairs};

    use super::*;

    #[tokio::test]
    #[test_log::test]
    async fn test_api_execute_wasm() {
        let wasm = include_bytes!("./hello.wasm");
        let session_key_pairs = SessionKeyPairs::default();

        let mut state = HypervisorState::default();
        state.set_session_key_pairs(session_key_pairs.clone());

        let server =
            axum_test::TestServer::new(Router::new().register_api(api_register).with_state(state))
                .unwrap();

        let sk = k256::ecdsa::SigningKey::random(&mut rand::rngs::OsRng);
        let user_pk = sk.verifying_key();

        let (session_pk, session_id) = session_key_pairs.create(user_pk);
        let cipher = crypto::create_encrypt_key(&sk, &session_pk, session_id).unwrap();

        let nonce = crypto::derive_msg_nonce(session_id);
        let encrypted_wasm = cipher.encrypt(&nonce, wasm.as_slice()).unwrap();
        let encrypted_arguments = { vec!["tress".to_string()].into_iter() }
            .map(|a| cipher.encrypt(&nonce, a.as_bytes()).unwrap())
            .map(const_hex::encode)
            .collect::<Vec<_>>();

        let response = server
            .post("/test/execute/wasm")
            .json(&ExecutionRequest {
                encrypted_wasm: const_hex::encode(&encrypted_wasm),
                encrypted_arguments: encrypted_arguments.clone(),
                public_key: crypto::pk_to_hex(user_pk),
            })
            .await;

        response.assert_status_ok();

        let result: ExecutionResponse = response.json();
        let result_nonce = *Nonce::from_slice(&const_hex::decode(result.result_nonce).unwrap());
        let result_commitment = commitment::build_result_commitment(
            user_pk,
            &session_pk,
            session_id,
            &const_hex::encode(encrypted_wasm),
            &encrypted_arguments,
            result_nonce,
            &result.encrypted_result,
        );
        assert_eq!(
            result_commitment.as_slice(),
            &const_hex::decode(result.result_commitment).unwrap(),
            "invalid commitment"
        );

        let output = cipher
            .decrypt(
                &result_nonce,
                { const_hex::decode(&result.encrypted_result).unwrap() }.as_slice(),
            )
            .unwrap();

        assert_eq!(String::from_utf8(output).unwrap(), "Hello tress\n");
    }
}
