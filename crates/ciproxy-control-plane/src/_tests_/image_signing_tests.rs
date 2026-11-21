//! Cloud CDN signature matches the independent official algorithm.

use super::{CdnSigning, ImageSigning};

#[test]
fn artifact_delivery_signs_exact_url_with_selected_key_and_expiry() {
    let signer = CdnSigning::new("test-key".to_owned(), "AAECAwQFBgcICQoLDA0ODw==").unwrap();
    let at = "2026-11-05T12:00:00Z".parse().unwrap();
    let signed = signer
        .sign("https://images.example/full/firecracker.tar.zst", at)
        .unwrap();
    assert_eq!(
        signed.expose_secret(),
        "https://images.example/full/firecracker.tar.zst?Expires=1793887200&KeyName=test-key&Signature=O04eFm3T9oydazR3cTo-1qhj_kY="
    );
}
