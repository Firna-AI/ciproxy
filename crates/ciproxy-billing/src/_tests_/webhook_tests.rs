//! The public webhook boundary rejects a bad signature before routing.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use unimock::{MockFn, Unimock, matching};

use crate::clock::ClockMock;
use crate::error::Error;
use crate::service::api::BillingApi;
use crate::service::runtime::{BillingService, Redirects};
use crate::webhook::SigningSecret;

#[test]
fn webhook_rejects_a_wrong_signature_even_for_an_ignored_event() {
    let now: DateTime<Utc> = "2026-11-04T12:00:00Z".parse().unwrap();
    let billing: Box<dyn BillingApi> = Box::new(BillingService::new(
        Arc::new(Unimock::new(())),
        Arc::new(Unimock::new(())),
        Arc::new(Unimock::new(
            ClockMock::now.each_call(matching!()).returns(now),
        )),
        SigningSecret::new("whsec_offline_boundary".into()),
        Redirects {
            success_url: "https://app.example/success".into(),
            cancel_url: "https://app.example/cancel".into(),
            portal_return_url: "https://app.example/billing".into(),
        },
    ));
    let raw = br#"{"id":"evt_ignored_bad_signature","created":1793793600,"type":"unrelated.event","data":null}"#;
    let signature =
        "t=1793793600,v1=0000000000000000000000000000000000000000000000000000000000000000";
    let result = billing.webhook(signature, raw);
    assert!(
        matches!(result, Err(Error::InvalidSignature)),
        "expected InvalidSignature, got {result:?}"
    );
}
