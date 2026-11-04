//! Signature verification protects the raw bytes, timestamp, and rotation.

use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

use crate::error::Error;
use crate::webhook::SigningSecret;

#[test]
fn signature_requires_exact_body_recent_timestamp_and_a_valid_rotation_signature() {
    let secret = SigningSecret::new("whsec_test_only".into());
    let raw = b"{\"id\":\"evt_one\"}";
    let mut mac = Hmac::<Sha256>::new_from_slice(b"whsec_test_only").unwrap();
    mac.update(b"1793793600.");
    mac.update(raw);
    let signature = format!(
        "t=1793793600,v1=bad,v1={}",
        hex::encode(mac.finalize().into_bytes())
    );
    assert!(secret.verify(&signature, raw, 1793793600).is_ok());
    assert!(matches!(
        secret.verify(&signature, b"{}", 1793793600),
        Err(Error::InvalidSignature)
    ));
    assert!(matches!(
        secret.verify(&signature, raw, 1793793901),
        Err(Error::InvalidSignature)
    ));
    assert!(matches!(
        secret.verify(&signature, raw, 1793793299),
        Err(Error::InvalidSignature)
    ));
    assert_eq!(format!("{secret:?}"), "SigningSecret([redacted])");
}
