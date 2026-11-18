//! Real provider throttles suspend whole recovery passes through their deadline.

use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use axum::http::StatusCode;
use chrono::DateTime;
use ciproxy_github::delivery_worker::{RecoveryPass, RecoveryWorker};
use ciproxy_github::redelivery::RecoverDeliveries;
use ciproxy_github::runtime::RuntimeMock;
use reqwest::Method;
use unimock::{MockFn, Unimock, matching};

use super::delivery_http::{Peer, Reply};
use super::delivery_tests::api;

#[tokio::test]
async fn recovery_worker_waits_for_http_throttle_deadlines_before_resuming_history() {
    for on_post in [false, true] {
        for (status, headers, due) in [
            (
                StatusCode::FORBIDDEN,
                vec![
                    ("retry-after", "125"),
                    ("x-ratelimit-remaining", "0"),
                    ("x-ratelimit-reset", "1793838000"),
                ],
                1_793_836_985,
            ),
            (
                StatusCode::TOO_MANY_REQUESTS,
                vec![
                    ("x-ratelimit-remaining", "0"),
                    ("x-ratelimit-reset", "1793837100"),
                ],
                1_793_837_100,
            ),
            (StatusCode::TOO_MANY_REQUESTS, vec![], 1_793_836_920),
        ] {
            let mut replies = Vec::new();
            if on_post {
                replies.push(Reply::new(
                    StatusCode::OK,
                    r#"[{"id":21,"guid":"retry-1","delivered_at":"2026-11-05T00:00:00Z","status_code":503,"event":"repository"},{"id":22,"guid":"retry-2","delivered_at":"2026-11-05T00:00:00Z","status_code":503,"event":"installation"}]"#,
                    &[],
                ));
            }
            replies.push(Reply::new(status, "", &headers));
            replies.push(Reply::new(StatusCode::OK, "[]", &[]));
            let peer = Peer::start(replies).await;
            let now = Arc::new(AtomicI64::new(1_793_836_860));
            let clock = now.clone();
            let clock = RuntimeMock::now
                .each_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    DateTime::from_timestamp(clock.load(Ordering::SeqCst), 0).unwrap()
                }));
            let runtime = Arc::new(if on_post {
                Unimock::new((
                    clock,
                    RuntimeMock::pause.each_call(matching!(_)).returns(()),
                ))
            } else {
                Unimock::new(clock)
            });
            let worker = RecoveryWorker::new(
                Arc::new(RecoverDeliveries {
                    api: Arc::new(api(&peer)),
                }),
                runtime,
            );
            assert!(matches!(worker.retry_failed().await, Ok(())));
            let first = peer.requests();
            assert_eq!(first.len(), if on_post { 2 } else { 1 });
            assert_eq!(
                first
                    .iter()
                    .filter(|(method, _)| *method == Method::POST)
                    .count(),
                usize::from(on_post),
            );
            now.store(due - 1, Ordering::SeqCst);
            assert!(matches!(worker.retry_failed().await, Ok(())));
            assert_eq!(peer.requests(), first);
            now.store(due, Ordering::SeqCst);
            assert!(matches!(worker.retry_failed().await, Ok(())));
            let mut resumed = first;
            resumed.push((
                Method::GET,
                "/api/v3/app/hook/deliveries?per_page=100".to_owned(),
            ));
            assert_eq!(peer.requests(), resumed);
        }
    }
}
