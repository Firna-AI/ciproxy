//! Exact webhook bytes must authenticate before installation processing.

use crate::app::webhook::{HmacVerifier, WebhookVerifier};
use crate::error::Error;
use crate::hooks::{AppWebhooks, WebhookHandler};
use std::sync::Arc;
use unimock::Unimock;

use super::support::app;

#[tokio::test]
async fn tampered_webhook_is_rejected_before_json_is_used() {
    let verifier = HmacVerifier::new("It's a Secret to Everybody".into());
    let signature = "sha256=757107ea0eb2509fc211221cce984b8a37570b6d7586c22c46f4379c8b043e17";
    verifier.verify(signature, b"Hello, World!").await.unwrap();
    assert!(matches!(
        verifier.verify(signature, b"Hello, world!").await,
        Err(Error::InvalidSignature)
    ));
    let handler = AppWebhooks {
        verifier: Arc::new(verifier),
        app: Arc::new(app(Unimock::new(()))),
    };
    assert!(matches!(
        handler
            .handle("installation", signature, b"{unreadable")
            .await,
        Err(Error::InvalidSignature)
    ));
}
