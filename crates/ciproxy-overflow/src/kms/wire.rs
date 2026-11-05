//! Private Cloud KMS JSON request and integrity envelopes.

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct EncryptRequest {
    pub(super) plaintext: String,
    pub(super) additional_authenticated_data: String,
    pub(super) plaintext_crc32c: String,
    pub(super) additional_authenticated_data_crc32c: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct EncryptResponse {
    pub(super) name: String,
    pub(super) ciphertext: String,
    pub(super) ciphertext_crc32c: String,
    pub(super) verified_plaintext_crc32c: bool,
    pub(super) verified_additional_authenticated_data_crc32c: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DecryptRequest {
    pub(super) ciphertext: String,
    pub(super) additional_authenticated_data: String,
    pub(super) ciphertext_crc32c: String,
    pub(super) additional_authenticated_data_crc32c: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DecryptResponse {
    pub(super) plaintext: String,
    pub(super) plaintext_crc32c: String,
}
