use aes_gcm_siv::{aead::Aead, Nonce};
use alloy::signers::local::PrivateKeySigner;
use hypervisor::{commitment, crypto, Config};
use reqwest::StatusCode;
use x402_reqwest::{MaxTokenAmountFromAmount, ReqwestWithPayments, ReqwestWithPaymentsBuild};
use x402_rs::network::{Network, USDCDeployment};

#[tokio::test]
#[test_log::test]
async fn test_x402_execute_wasm() {
    let hype = hypervisor::Server::build(Config::default()).unwrap();
    tokio::spawn(hype.start());

    let wasm = include_bytes!("./hello.wasm");

    let sk = k256::ecdsa::SigningKey::random(&mut rand::rngs::OsRng);
    let user_pk = sk.verifying_key();

    let client = reqwest::Client::new();
    let resp = client
        .post("http://localhost:3000/encrypt/create_keypair")
        .json(&hypervisor::api::encrypt::CreateKeyPairRequest {
            pubkey: crypto::pk_to_hex(user_pk),
        })
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = resp
        .json::<hypervisor::api::encrypt::CreateKeyPairResponse>()
        .await
        .unwrap();

    let session_pk = crypto::pk_from_hex(&resp.session_pubkey).unwrap();
    let session_id = resp.session_id;

    let cipher = crypto::create_encrypt_key(&sk, &session_pk, session_id).unwrap();

    let nonce = crypto::derive_msg_nonce(session_id);
    let encrypted_wasm = cipher.encrypt(&nonce, wasm.as_slice()).unwrap();
    let encrypted_arguments = { vec!["tress".to_string()].into_iter() }
        .map(|a| cipher.encrypt(&nonce, a.as_bytes()).unwrap())
        .map(const_hex::encode)
        .collect::<Vec<_>>();

    // Test account
    let test_payer: PrivateKeySigner =
        "0xbf50d0452bfa77ff2ab9f396d7ff67e1ffeda0c401eb0d71235fca9fee691009"
            .parse()
            .unwrap();

    let payment = USDCDeployment::by_network(Network::BaseSepolia);
    let x402_client = reqwest::Client::new()
        .with_payments(test_payer)
        .prefer(payment.clone())
        .max(payment.amount(0.02).unwrap())
        .build();

    let response = x402_client
        .post("http://localhost:3000/x402_execute/test/wasm")
        .json(&hypervisor::api::execute::wasm::ExecutionRequest {
            encrypted_wasm: const_hex::encode(&encrypted_wasm),
            encrypted_arguments: encrypted_arguments.clone(),
            public_key: crypto::pk_to_hex(user_pk),
        })
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let result = response
        .json::<hypervisor::api::execute::wasm::ExecutionResponse>()
        .await
        .unwrap();

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
