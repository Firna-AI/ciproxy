//! Construct the blocking Stripe client outside the asynchronous startup worker.

use std::sync::Arc;

use ciproxy_billing::clock::SystemClock;
use ciproxy_billing::service::runtime::{BillingService, Redirects};
use ciproxy_billing::webhook::SigningSecret;
use ciproxy_store_interface::store::Store;
use secrecy::ExposeSecret;

use crate::config::Config;
use crate::error::Result;
use crate::pages::{PageComposition, StripeSettings};

pub(crate) async fn compose(config: &Config, store: Arc<dyn Store>) -> Result<Arc<BillingService>> {
    let key = config.secret("stripe-secret-key")?;
    let secret = config.secret("stripe-webhook-secret")?;
    let origin = config.public_url.trim_end_matches('/');
    let redirects = Redirects {
        success_url: format!("{origin}/billing/return?session_id={{CHECKOUT_SESSION_ID}}"),
        cancel_url: format!("{origin}/plans"),
        portal_return_url: format!("{origin}/billing"),
    };
    PageComposition::billing(
        store,
        Arc::new(SystemClock),
        StripeSettings {
            endpoint: "https://api.stripe.com/".into(),
            secret_key: key,
            signing_secret: SigningSecret::new(secret.expose_secret().to_owned()),
            redirects,
        },
    )
    .await
}
