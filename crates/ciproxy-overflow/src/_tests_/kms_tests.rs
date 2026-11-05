//! Real cipher requests authenticate workspace, identity, purpose, and checksums.

use std::sync::Arc;

use base64::{Engine, engine::general_purpose::STANDARD};
use unimock::{MockFn, Unimock, matching};

use crate::crypto::{KeyCipher, KeyContext, SecretPurpose};
use crate::error::Error;
use crate::kms::cipher::CloudKmsCipher;
use crate::kms::transport::{KmsAction, KmsTransportMock};
use crate::secrets::SecretBytes;

use uuid::Uuid;

fn id(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

#[tokio::test]
async fn kms_encrypt_binds_workspace_identity_and_purpose_and_checks_ciphertext() {
    let transport = Unimock::new((
        KmsTransportMock::post.some_call(matching!(KmsAction::Encrypt, _)).answers(&|_, _, payload| {
            let request: serde_json::Value = serde_json::from_slice(payload.expose()).unwrap();
            assert_eq!(request["plaintext"], "Ym9hdC1rZXk=");
            assert_eq!(STANDARD.decode(request["additionalAuthenticatedData"].as_str().unwrap()).unwrap(),
                b"ciproxy-overflow/v1/00000000-0000-0000-0000-000000000001/00000000-0000-0000-0000-000000000002/boat-key");
            assert_eq!(request["plaintextCrc32c"], crc32c::crc32c(b"boat-key").to_string());
            Ok(SecretBytes::new(serde_json::to_vec(&serde_json::json!({ "name": "projects/p/locations/l/keyRings/r/cryptoKeys/k/cryptoKeyVersions/1", "ciphertext": "c2VhbGVk", "ciphertextCrc32c": crc32c::crc32c(b"sealed").to_string(), "verifiedPlaintextCrc32c": true, "verifiedAdditionalAuthenticatedDataCrc32c": true })).unwrap()))
        }),
        KmsTransportMock::owns_version.some_call(matching!("projects/p/locations/l/keyRings/r/cryptoKeys/k/cryptoKeyVersions/1")).returns(true),
    ));
    let cipher: &dyn KeyCipher = &CloudKmsCipher::new(Arc::new(transport));
    let ciphertext = cipher
        .seal(
            KeyContext {
                workspace_id: id(1),
                id: id(2),
                purpose: SecretPurpose::BoatKey,
            },
            SecretBytes::new(b"boat-key".to_vec()),
        )
        .await
        .unwrap();
    assert_eq!(ciphertext, b"sealed");
}

#[tokio::test]
async fn kms_decrypt_rejects_corrupted_plaintext_and_keeps_launch_scope() {
    let transport = Unimock::new(KmsTransportMock::post.some_call(matching!(KmsAction::Decrypt, _)).answers(&|_, _, payload| {
        let request: serde_json::Value = serde_json::from_slice(payload.expose()).unwrap();
        assert_eq!(STANDARD.decode(request["additionalAuthenticatedData"].as_str().unwrap()).unwrap(),
            b"ciproxy-overflow/v1/00000000-0000-0000-0000-000000000001/00000000-0000-0000-0000-00000000000b/launch");
        Ok(SecretBytes::new(br#"{"plaintext":"Y29ycnVwdA==","plaintextCrc32c":"0"}"#.to_vec()))
    }));
    let cipher: &dyn KeyCipher = &CloudKmsCipher::new(Arc::new(transport));
    assert!(matches!(
        cipher
            .open(
                KeyContext {
                    workspace_id: id(1),
                    id: id(11),
                    purpose: SecretPurpose::Launch
                },
                b"sealed".to_vec()
            )
            .await,
        Err(Error::KmsIntegrity)
    ));
}
