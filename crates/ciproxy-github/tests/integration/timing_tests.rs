//! Timing lookup reads all attempts and pages, then requires a unique runner.

use std::sync::Arc;
use std::sync::atomic::AtomicI64;

use ciproxy_github::auth::tokens::RotatingTokens;
use ciproxy_github::rest::GitHubRest;
use ciproxy_github::rest::http::RestClient;
use ciproxy_store_interface::github_timing::JobTimes;
use reqwest::{Client, Url};
use serde_json::json;

use super::fake_http::Server;
use super::support::{FixedJwt, FixedRuntime};

#[tokio::test]
async fn timing_lookup_pages_all_attempts_and_matches_exact_run_and_unique_runner() {
    let server = Server::start(None).await;
    let http = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let api = Url::parse(&format!("{}/api/v3/", server.origin)).unwrap();
    let tokens = Arc::new(RotatingTokens::new(
        http.clone(),
        api.clone(),
        Arc::new(FixedJwt),
        Arc::new(FixedRuntime(AtomicI64::new(1_793_793_600))),
    ));
    let rest = RestClient::new(http, api, tokens);
    *server.state.job_pages.lock().unwrap() = vec![
        json!({"total_count":3,"jobs":[
            {"id":331,"run_id":99,"runner_id":71,"started_at":"2026-11-04T12:00:00Z","completed_at":null,"status":"completed"},
            {"id":332,"run_id":31,"runner_id":72,"started_at":null,"completed_at":null,"status":"in_progress"}]}),
        json!({"total_count":3,"jobs":[
            {"id":333,"run_id":31,"runner_id":71,"started_at":"2026-11-04T12:00:00.000000001Z","completed_at":"2026-11-04T12:01:00.000000002Z","status":"completed"}]}),
    ];
    assert_eq!(
        rest.job_times(19, "customer/private", 31, 71)
            .await
            .unwrap(),
        Some(JobTimes {
            actions_job_id: 333,
            started_at: Some("2026-11-04T12:00:00.000000001Z".parse().unwrap()),
            completed_at: Some("2026-11-04T12:01:00.000000002Z".parse().unwrap()),
            terminal: true,
        })
    );
    assert_eq!(
        server
            .state
            .requests
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, path, _)| path.contains("/jobs?"))
            .map(|(_, path, _)| path.clone())
            .collect::<Vec<_>>(),
        vec![
            "/api/v3/repos/customer/private/actions/runs/31/jobs?filter=all&per_page=100&page=1",
            "/api/v3/repos/customer/private/actions/runs/31/jobs?filter=all&per_page=100&page=2",
        ]
    );
    server.state.job_pages.lock().unwrap()[0]["jobs"][0]["run_id"] = json!(31);
    assert_eq!(
        rest.job_times(19, "customer/private", 31, 71)
            .await
            .unwrap(),
        None
    );
}
