//! Synthetic mounted credentials are local to one executable scenario.

use std::path::Path;
use std::process::Stdio;

use tokio::process::Command;

/// Provide synthetic key bytes for the real CDN signing client.
pub(super) fn cdn(directory: &Path) {
    std::fs::write(
        directory.join("cdn-signing-key"),
        "AAECAwQFBgcICQoLDA0ODw==",
    )
    .unwrap();
}

/// Generate a disposable RSA key and the other mounted startup values.
pub(super) async fn serving(directory: &Path) {
    for (name, value) in [
        ("github-webhook-secret", "synthetic-webhook-secret"),
        ("github-oauth-secret", "synthetic-oauth-secret"),
        ("stripe-secret-key", "sk_test_synthetic_shutdown"),
        ("stripe-webhook-secret", "whsec_synthetic_shutdown"),
        (
            "session-key",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        ),
    ] {
        std::fs::write(directory.join(name), value).unwrap();
    }
    let status = Command::new("openssl")
        .args([
            "genpkey",
            "-algorithm",
            "RSA",
            "-pkeyopt",
            "rsa_keygen_bits:2048",
            "-out",
        ])
        .arg(directory.join("github-app-private-key"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .expect("OpenSSL must generate the synthetic GitHub App key");
    assert_eq!(status.code(), Some(0), "generate a parseable test RSA key");
}
