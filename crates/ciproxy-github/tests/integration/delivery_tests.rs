//! Recovery uses the App JWT and complete cursor history through real HTTP.

use std::sync::Arc;

use axum::http::StatusCode;
use chrono::DateTime;
use ciproxy_github::delivery_worker::{RecoveryPass, RecoveryWorker};
use ciproxy_github::redelivery::{HttpDeliveries, RecoverDeliveries};
use ciproxy_github::runtime::RuntimeMock;
use reqwest::redirect::Policy;
use reqwest::{Client, Method, Url};
use unimock::{MockFn, Unimock, matching};

use super::delivery_http::{Peer, Reply};
use super::support::FixedJwt;

pub(super) fn api(peer: &Peer) -> HttpDeliveries {
    HttpDeliveries {
        http: Client::builder().redirect(Policy::none()).build().unwrap(),
        api: Url::parse(&format!("{}/api/v3/", peer.origin)).unwrap(),
        signer: Arc::new(FixedJwt),
    }
}

#[tokio::test]
async fn recovery_reads_last_cursor_page_with_app_jwt_and_redelivers_its_failed_delivery() {
    let peer = Peer::start(vec![
        Reply::new(
            StatusCode::OK,
            r#"[{"id":11,"guid":"accepted","delivered_at":"2026-11-05T12:00:00Z","status_code":204,"event":"installation"}]"#,
            &[(
                "link",
                "<https://api.github.test/app/hook/deliveries?per_page=100&cursor=page%2Ftwo%2B%3D>; rel=\"next\"",
            )],
        ),
        Reply::new(
            StatusCode::OK,
            r#"[{"id":21,"guid":"retry","delivered_at":"2026-11-05T11:00:00Z","status_code":503,"event":"repository"}]"#,
            &[(
                "link",
                "<https://api.github.test/app/hook/deliveries?cursor=page%2Fone>; rel=\"prev\"",
            )],
        ),
        Reply::new(StatusCode::ACCEPTED, "", &[]),
    ])
    .await;
    let recovery = RecoveryWorker::new(
        Arc::new(RecoverDeliveries {
            api: Arc::new(api(&peer)),
        }),
        Arc::new(Unimock::new((
            RuntimeMock::now
                .each_call(matching!())
                .returns(DateTime::from_timestamp(1_793_880_300, 0).unwrap()),
            RuntimeMock::pause.each_call(matching!(_)).returns(()),
        ))),
    );
    assert!(matches!(recovery.retry_failed().await, Ok(())));
    assert_eq!(
        peer.requests(),
        vec![
            (
                Method::GET,
                "/api/v3/app/hook/deliveries?per_page=100".to_owned(),
            ),
            (
                Method::GET,
                "/api/v3/app/hook/deliveries?per_page=100&cursor=page%2Ftwo%2B%3D".to_owned(),
            ),
            (
                Method::POST,
                "/api/v3/app/hook/deliveries/21/attempts".to_owned(),
            ),
        ]
    );
}
