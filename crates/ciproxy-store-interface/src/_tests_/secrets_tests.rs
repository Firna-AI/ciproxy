//! Diagnostic redaction of stored hashes and ciphertext.

use crate::secrets::{EncryptedBoatKey, TokenHash};

#[test]
fn secret_debug_output_redacts_stored_payloads() {
    let hash = TokenHash::new(vec![11, 22, 33]);
    let key = EncryptedBoatKey::new(vec![44, 55, 66]);
    assert_eq!(format!("{hash:?}"), "TokenHash([redacted])");
    assert_eq!(format!("{key:?}"), "EncryptedBoatKey([redacted])");
}
