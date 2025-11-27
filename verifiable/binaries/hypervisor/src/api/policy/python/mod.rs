use aes_gcm_siv::aead::Aead;
use anyhow::{anyhow, Context};
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::info;
use uuid::Uuid;

use crate::{
    error::HypervisorError,
    types::HypervisorState,
    utils::{
        self, crypto,
        x402::{self, X402_FIXED_PRICE_USDC},
    },
};

pub(crate) fn api_register(router: Router<HypervisorState>) -> Router<HypervisorState> {
    router
        .route("/test/policy/unsafe/python", post(execute_python))
        .route(
            "/test/policy/unsafe/python/attest",
            post(attest_execute_python),
        )
}

pub(crate) fn api_x402_register(
    router: Router<HypervisorState>,
    state: HypervisorState,
) -> Router<HypervisorState> {
    let x402_router = Router::new()
        .route("/unsafe/python", post(execute_python))
        .route("/unsafe/python/attest", post(attest_execute_python))
        .layer(x402::create_x402_middleware(X402_FIXED_PRICE_USDC))
        .with_state(state);

    router.nest("/x402_policy", x402_router)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifiableExecutionResponse {
    pub session_id: Uuid,
    pub msg_nonce: String,
    pub encrypted_result: String,
    pub result_commitment: String,
    pub result_quote: String,
}

async fn attest_execute_python(
    state: State<HypervisorState>,
    req: Json<ExecutionRequest>,
) -> Result<Json<VerifiableExecutionResponse>, HypervisorError> {
    let Json(resp) = execute_python(state, req).await?;
    let commitment: [u8; 32] =
        const_hex::decode_to_array(&resp.result_commitment).expect("impossible");

    let quote = attest::get_quote(utils::attest::generate_raw_report_from_hash(commitment))
        .context("get execute result quote")
        .context(StatusCode::INTERNAL_SERVER_ERROR)?;

    let verifiable_resp = VerifiableExecutionResponse {
        session_id: resp.session_id,
        msg_nonce: resp.msg_nonce,
        encrypted_result: resp.encrypted_result,
        result_commitment: resp.result_commitment,
        result_quote: const_hex::encode(quote.to_bytes()),
    };

    Ok(Json(verifiable_resp))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub encrypted_python: String,
    #[serde(default = "Vec::new")]
    pub encrypted_arguments: Vec<String>,
    pub public_key: String,
}

#[derive(Debug, Serialize)]
pub struct ExecutionResponse {
    pub session_id: Uuid,
    pub msg_nonce: String,
    pub encrypted_result: String,
    pub result_commitment: String,
}

#[tracing::instrument(skip(state, req), err)]
async fn execute_python(
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

    let decrypted_python = {
        let encrypted_bytes = const_hex::decode(&req.encrypted_python)
            .context("invalid python binary hex")
            .context(StatusCode::BAD_REQUEST)?;

        let decrypted = cipher
            .decrypt(&msg_nonce, encrypted_bytes.as_slice())
            .map_err(|e| anyhow!(e.to_string()))
            .context("decrypt python binary")
            .context(StatusCode::BAD_REQUEST)?;

        String::from_utf8(decrypted)
            .context("invalid python string")
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
        "processing python execution request"
    );

    let mut child = Command::new("python")
        .arg("-")
        .args(decrypted_arguments)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("execute python")
        .context(StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(mut stdin) = child.stdin.take() {
        { stdin.write_all(decrypted_python.as_bytes()).await }
            .context("pass python script")
            .context(StatusCode::INTERNAL_SERVER_ERROR)?;

        { stdin.shutdown().await }
            .context("close stdin")
            .context(StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let output = child
        .wait_with_output()
        .await
        .context("wait python output")
        .context(StatusCode::BAD_REQUEST)?;

    let app_output = if output.status.success() {
        String::from_utf8_lossy(&output.stdout)
    } else {
        String::from_utf8_lossy(&output.stderr)
    };

    info!(
        session_id = %session_id,
        public_key = req.public_key,
        execution_time_ms = start_time.elapsed().as_millis(),
        status = "success",
        msg = "python execution completed successfully"
    );

    let output_nonce = crypto::derive_msg_nonce(app_output.as_bytes());
    let encrypted_result = {
        let encrypted = cipher
            .encrypt(&output_nonce, app_output.as_bytes())
            .map_err(|e| anyhow!(e.to_string()))?;

        const_hex::encode(encrypted)
    };

    let result_commitment = {
        let mut entries = vec![
            user_pk.to_encoded_point(true).to_bytes(),
            session_sk.verifying_key().to_encoded_point(true).to_bytes(),
            Box::new(*session_id.as_bytes()),
            req.encrypted_python.as_bytes().into(),
        ];

        entries.extend(
            { req.encrypted_arguments.iter().cloned() }
                .map(|a| a.as_bytes().into())
                .collect::<Vec<_>>(),
        );

        entries.extend([
            output_nonce.to_vec().into(),
            encrypted_result.as_bytes().into(),
        ]);

        const_hex::encode(utils::hasher::hash_multi(&entries))
    };

    let resp = ExecutionResponse {
        session_id,
        msg_nonce: const_hex::encode(output_nonce),
        encrypted_result,
        result_commitment,
    };

    Ok(axum::Json(resp))
}

/// Validate execution request
fn validate_execution_request(request: &ExecutionRequest) -> Result<(), HypervisorError> {
    let validate = || -> anyhow::Result<()> {
        // Validate encrypted_python
        anyhow::ensure!(
            !request.encrypted_python.trim().is_empty(),
            "encrypted_python cannot be empty"
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

#[cfg(test)]
mod tests {
    use aes_gcm_siv::aead::Aead;

    use crate::utils::crypto;
    use crate::{api::RouterRegister, types::SessionKeyPairs};

    use super::*;

    #[tokio::test]
    #[test_log::test]
    async fn test_api_execute_python() {
        let python = include_bytes!("./hello.py");
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
        let encrypted_python = cipher.encrypt(&nonce, python.as_slice()).unwrap();
        let encrypted_arguments = { vec!["tress".to_string()].into_iter() }
            .map(|a| cipher.encrypt(&nonce, a.as_bytes()).unwrap())
            .map(const_hex::encode)
            .collect();

        let response = server
            .post("/test/policy/unsafe/python")
            .json(&ExecutionRequest {
                encrypted_python: const_hex::encode(encrypted_python),
                encrypted_arguments,
                public_key: crypto::pk_to_hex(user_pk),
            })
            .await;

        response.assert_status_ok();
        println!("response {}", response.text());
    }
}
