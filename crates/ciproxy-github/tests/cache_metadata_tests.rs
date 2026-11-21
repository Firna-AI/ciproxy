//! Trusted cache listing pagination and timestamp conversion through real HTTP.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use axum::extract::State;
use axum::http::{HeaderMap, Uri};
use axum::{Json, Router};
use ciproxy_github::auth::InstallationTokens;
use ciproxy_github::error::Result;
use ciproxy_github::rest::cache_metadata::CacheEntries;
use ciproxy_github::rest::http::RestClient;
use reqwest::{Client, Url};
use secrecy::SecretString;
use serde_json::{Value, json};
use tokio::net::TcpListener;

struct SyntheticTokens;

#[async_trait]
impl InstallationTokens for SyntheticTokens {
    async fn token(&self, installation_id: i64) -> Result<SecretString> {
        assert_eq!(installation_id, 19);
        Ok("synthetic-app-token".into())
    }
    async fn invalidate(&self, _: i64) {}
}

#[tokio::test]
async fn cache_metadata_reads_every_page_and_current_entries() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let api = Url::parse(&format!("http://{}/", listener.local_addr().unwrap())).unwrap();
    let revision = Arc::new(AtomicU64::new(0));
    let router = Router::new().fallback(listing).with_state(revision.clone());
    let task = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let rest = RestClient::new(
        Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap(),
        api,
        Arc::new(SyntheticTokens),
    );
    let first = rest.cache_entries(19, "org/repo").await.unwrap();
    assert_eq!(first.len(), 101);
    assert_eq!(first[100].id, "101");
    assert_eq!(first[100].git_ref.0, "refs/heads/feature");
    assert_eq!(first[100].created_at, "1794096000123456789");
    revision.store(1, Ordering::SeqCst);
    let second = rest.cache_entries(19, "org/repo").await.unwrap();
    assert_eq!(second[100].key, "new-during-job");
    task.abort();
}

async fn listing(
    State(revision): State<Arc<AtomicU64>>,
    headers: HeaderMap,
    uri: Uri,
) -> Json<Value> {
    assert_eq!(headers["authorization"], "Bearer synthetic-app-token");
    assert_eq!(uri.path(), "/repos/org/repo/actions/caches");
    let page = if uri.query() == Some("per_page=100&page=1&sort=created_at&direction=asc") {
        1
    } else {
        2
    };
    let entries: Vec<_> = if page == 1 {
        (1..=100).map(|id| entry(id, "deps")).collect()
    } else {
        vec![entry(
            101,
            if revision.load(Ordering::SeqCst) == 0 {
                "existing"
            } else {
                "new-during-job"
            },
        )]
    };
    Json(json!({"total_count":101,"actions_caches":entries}))
}

fn entry(id: u64, key: &str) -> Value {
    json!({"id":id,"ref":"refs/heads/feature","key":key,"version":"v1",
        "created_at":"2026-11-08T00:00:00.123456789Z"})
}
