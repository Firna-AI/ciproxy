//! Running proxy recovery stays retryable and preserves the Docker outcome.

use ciproxy_protocol::cache::CacheConnection;
use ciproxy_protocol::guest::JobPreparation;
use ciproxy_protocol::secret::Secret;
use unimock::{MockFn, Unimock, matching};

use crate::error::Error;
use crate::handoff::PreparedProxy;
use crate::preparation::{Prepared, PreparedJob};
use crate::proxy::ProxySessionMock;

fn connection(token: &str) -> CacheConnection {
    CacheConnection {
        url: Secret::new("http://machine:7887"),
        token: Secret::new(token),
    }
}

#[tokio::test]
async fn selected_proxy_retries_credentials_without_repeating_docker_preparation() {
    let proxy = Unimock::new(ProxySessionMock::replace_connection.stub(|each| {
        each.call(matching!((replacement,) if replacement == &connection("new")))
            .answers(&|_, _| Err(Error::PoweroffFailed))
            .once()
            .then()
            .answers(&|_, _| Ok(()))
            .once();
    }));
    let expected = JobPreparation {
        cache_ready: true,
        docker_attached: true,
    };
    let mut prepared = Prepared {
        initial: PreparedProxy {
            proxy: Some(Box::new(proxy)),
            outcome: expected,
        },
        connection: Some(connection("old")),
    };
    assert_eq!(prepared.replay(Some(connection("old"))).await, expected);
    assert_eq!(prepared.replay(None).await, expected);
    assert_eq!(prepared.replay(Some(connection("new"))).await, expected);
    assert_eq!(prepared.connection, Some(connection("old")));
    assert_eq!(prepared.replay(Some(connection("new"))).await, expected);
    assert_eq!(prepared.connection, Some(connection("new")));
    assert_eq!(prepared.replay(Some(connection("new"))).await, expected);
}

#[tokio::test]
async fn worker_github_fallback_cannot_be_changed_after_action_setup() {
    let mut prepared = Prepared {
        initial: PreparedProxy {
            proxy: None,
            outcome: JobPreparation {
                cache_ready: false,
                docker_attached: true,
            },
        },
        connection: Some(connection("old")),
    };
    assert_eq!(
        prepared.replay(Some(connection("new"))).await,
        JobPreparation {
            cache_ready: false,
            docker_attached: true
        }
    );
    assert_eq!(prepared.connection, Some(connection("old")));
}
