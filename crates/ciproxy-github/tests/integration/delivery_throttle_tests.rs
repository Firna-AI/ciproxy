//! Both delivery HTTP operations preserve provider throttle instructions.

use axum::http::StatusCode;
use ciproxy_github::error::Error;
use ciproxy_github::redelivery::DeliveryApi;

use super::delivery_http::{Peer, Reply};
use super::delivery_tests::api;

#[derive(Debug, Eq, PartialEq)]
enum Failure {
    Throttled(Option<u64>, Option<i64>),
    Access(u16),
}

fn failure(error: Error) -> Failure {
    match error {
        Error::DeliveryThrottled {
            retry_after_seconds,
            reset_at,
        } => Failure::Throttled(retry_after_seconds, reset_at.map(|time| time.timestamp())),
        Error::AccessUnavailable(status) => Failure::Access(status),
        unexpected => panic!("unexpected delivery failure: {unexpected:?}"),
    }
}

#[tokio::test]
async fn delivery_http_preserves_403_and_429_throttle_headers_for_list_and_post() {
    for (status, headers, expected) in [
        (
            StatusCode::FORBIDDEN,
            vec![("retry-after", "123")],
            Failure::Throttled(Some(123), None),
        ),
        (
            StatusCode::FORBIDDEN,
            vec![
                ("x-ratelimit-remaining", "0"),
                ("x-ratelimit-reset", "1797937200"),
            ],
            Failure::Throttled(None, Some(1_797_937_200)),
        ),
        (
            StatusCode::TOO_MANY_REQUESTS,
            vec![
                ("retry-after", "45"),
                ("x-ratelimit-remaining", "0"),
                ("x-ratelimit-reset", "1797937200"),
            ],
            Failure::Throttled(Some(45), Some(1_797_937_200)),
        ),
        (
            StatusCode::TOO_MANY_REQUESTS,
            vec![],
            Failure::Throttled(None, None),
        ),
        (
            StatusCode::TOO_MANY_REQUESTS,
            vec![
                ("retry-after", "invalid"),
                ("x-ratelimit-remaining", "0"),
                ("x-ratelimit-reset", "invalid"),
            ],
            Failure::Throttled(None, None),
        ),
        (StatusCode::FORBIDDEN, vec![], Failure::Access(403)),
    ] {
        let peer = Peer::start(vec![
            Reply::new(status, "", &headers),
            Reply::new(status, "", &headers),
        ])
        .await;
        let api = api(&peer);
        assert_eq!(
            failure(api.list(None).await.err().unwrap()),
            expected,
            "list with {status} and {headers:?}",
        );
        assert_eq!(
            failure(api.redeliver(21).await.unwrap_err()),
            expected,
            "POST with {status} and {headers:?}",
        );
    }
}
