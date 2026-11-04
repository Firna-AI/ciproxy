//! Rotating token permissions and group requests cross actual HTTP serialization.

use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use ciproxy_github::auth::InstallationTokens;
use ciproxy_github::auth::tokens::RotatingTokens;
use ciproxy_github::rest::GitHubRest;
use ciproxy_github::rest::http::RestClient;
use reqwest::redirect::Policy;
use reqwest::{Client, Method, Url};
use serde_json::json;

use super::fake_http::Server;
use super::support::{FixedJwt, FixedRuntime};

#[tokio::test]
async fn installation_tokens_rotate_and_group_payload_restricts_selected_private_repositories() {
    let server = Server::start(None).await;
    let http = Client::builder().redirect(Policy::none()).build().unwrap();
    let api = Url::parse(&format!("{}/api/v3/", server.origin)).unwrap();
    let runtime = Arc::new(FixedRuntime(AtomicI64::new(1_793_793_600)));
    let tokens = Arc::new(RotatingTokens::new(
        http.clone(),
        api.clone(),
        Arc::new(FixedJwt),
        runtime.clone(),
    ));
    tokens.token(19).await.unwrap();
    tokens.token(19).await.unwrap();
    assert_eq!(server.state.tokens.load(Ordering::SeqCst), 1);
    runtime.0.fetch_add(600, Ordering::SeqCst);
    tokens.token(19).await.unwrap();
    assert_eq!(server.state.tokens.load(Ordering::SeqCst), 2);
    let rest = RestClient::new(http, api, tokens);
    server
        .state
        .reject_repository_token
        .store(true, Ordering::SeqCst);
    let repositories = rest.repositories(19).await.unwrap();
    assert_eq!(server.state.tokens.load(Ordering::SeqCst), 3);
    assert_eq!(
        repositories
            .iter()
            .map(|repository| (
                &*repository.full_name,
                &*repository.default_branch,
                repository.private
            ))
            .collect::<Vec<_>>(),
        vec![
            ("customer/private", "trunk", true),
            ("customer/public", "main", false)
        ]
    );
    assert_eq!(
        rest.restrict_runner_group(19, "customer", "ciproxy-test", &[23], None)
            .await
            .unwrap(),
        47
    );
    let requests = server.state.requests.lock().unwrap();
    let token_requests: Vec<_> = requests
        .iter()
        .filter(|(_, path, _)| path.ends_with("access_tokens"))
        .collect();
    assert_eq!(
        token_requests[0].2,
        json!({"permissions":{"organization_self_hosted_runners":"write","metadata":"read","actions":"read"}})
    );
    let created = requests
        .iter()
        .find(|(method, path, _)| *method == Method::POST && path.ends_with("runner-groups"))
        .unwrap();
    assert_eq!(
        created.2,
        json!({"name":"ciproxy-test","visibility":"selected","allows_public_repositories":false,"selected_repository_ids":[23]})
    );
    let selected = requests
        .iter()
        .find(|(method, path, _)| *method == Method::PUT && path.ends_with("47/repositories"))
        .unwrap();
    assert_eq!(selected.2, json!({"selected_repository_ids":[23]}));
}
