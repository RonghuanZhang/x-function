use aes_gcm_siv::Nonce;
use k256::ecdsa::VerifyingKey;
use uuid::Uuid;

use crate::utils;

pub fn build_result_commitment(
    user_pk: &VerifyingKey,
    session_pk: &VerifyingKey,
    session_id: Uuid,
    encrypted_wasm: &str,
    encrypted_arguments: &[String],
    output_nonce: Nonce,
    encrypted_result: &String,
) -> [u8; 32] {
    let mut entries = vec![
        user_pk.to_encoded_point(true).to_bytes(),
        session_pk.to_encoded_point(true).to_bytes(),
        Box::new(*session_id.as_bytes()),
        encrypted_wasm.as_bytes().into(),
    ];

    entries.extend(
        { encrypted_arguments.iter().cloned() }
            .map(|a| a.as_bytes().into())
            .collect::<Vec<_>>(),
    );

    entries.extend([
        output_nonce.to_vec().into(),
        encrypted_result.as_bytes().into(),
    ]);

    utils::hasher::hash_multi(&entries)
}
