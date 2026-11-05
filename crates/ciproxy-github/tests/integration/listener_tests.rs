//! The real SDK, listener, and store process fake HTTP end to end.

use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use ciproxy_github::auth::tokens::RotatingTokens;
use ciproxy_github::listener::{DurableListener, ListenerOptions, ScaleSetListener};
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::rest::http::RestClient;
use ciproxy_store_interface::github::DemandPhase;
use reqwest::redirect::Policy;
use reqwest::{Client, Method, Url};
use tokio::sync::watch;

use super::database::Database;
use super::fake_http::Server;
use super::support::{Factory, FixedJwt, FixedRuntime, Output, setup};

#[tokio::test]
async fn http_listener_replays_uuid_jobs_without_duplicate_demand_or_terminal_regression() {
    let database = Database::new();
    let target = setup(&*database.store);
    let server = Server::start(Some(database.store.clone())).await;
    server.state.fail_later_facts.store(true, Ordering::SeqCst);
    let http = Client::builder().redirect(Policy::none()).build().unwrap();
    let api = Url::parse(&format!("{}/api/v3/", server.origin)).unwrap();
    let runtime = Arc::new(FixedRuntime(AtomicI64::new(1_793_793_600)));
    let tokens = Arc::new(RotatingTokens::new(
        http.clone(),
        api.clone(),
        Arc::new(FixedJwt),
        runtime.clone(),
    ));
    let output = Arc::new(Output::default());
    let listener = DurableListener {
        store: Arc::new(StoreAdapter::new(database.store.clone(), runtime)),
        rest: Arc::new(RestClient::new(http, api, tokens)),
        scale_sets: Arc::new(Factory(server.origin.clone())),
        output: output.clone(),
    };
    let (_stop, signal) = watch::channel(false);
    let result = listener
        .run(
            target,
            ListenerOptions {
                owner: "integration".to_owned(),
                capacity: 2,
                max_polls: Some(5),
            },
            signal,
        )
        .await;
    assert!(
        matches!(result, Ok(())),
        "known completion must survive loss of repository API access"
    );
    assert_eq!(server.state.facts_reads.load(Ordering::SeqCst), 1);
    let saved = database
        .store
        .github_jobs(target.workspace_id, target.organization_id)
        .unwrap();
    assert_eq!(saved.len(), 1);
    let job = &saved[0];
    assert_eq!(job.id.to_string(), "01a277ed-2171-562e-8431-03d0033029ab");
    assert_eq!(job.request_id, 0);
    assert_eq!(job.actions_job_id, None);
    assert_eq!(job.phase, DemandPhase::Completed);
    assert_eq!(
        job.assigned_at.unwrap().to_rfc3339(),
        "2026-11-04T10:46:21.620395766+00:00"
    );
    assert_eq!(
        job.started_at.unwrap().to_rfc3339(),
        "2026-11-04T10:46:21.661245605+00:00"
    );
    assert_eq!(
        job.completed_at.unwrap().to_rfc3339(),
        "2026-11-04T10:46:24.317465059+00:00"
    );
    assert_eq!(job.facts.event, "pull_request_target");
    assert_eq!(job.facts.head_repository, None);
    assert_eq!(job.facts.head_branch.as_deref(), Some("untrusted-feature"));
    assert_eq!(
        output
            .0
            .lock()
            .unwrap()
            .iter()
            .map(|job| job.phase)
            .collect::<Vec<_>>(),
        vec![
            DemandPhase::Assigned,
            DemandPhase::Started,
            DemandPhase::Completed
        ]
    );
    assert_eq!(
        *server.state.acknowledgements.lock().unwrap(),
        vec![
            DemandPhase::Assigned,
            DemandPhase::Assigned,
            DemandPhase::Started,
            DemandPhase::Completed,
            DemandPhase::Completed
        ]
    );
    let requests = server.state.requests.lock().unwrap();
    assert_eq!(
        requests
            .iter()
            .filter(|(_, path, _)| path.contains("acquirejobs"))
            .count(),
        0
    );
    assert_eq!(
        requests
            .iter()
            .filter(|(method, path, _)| *method == Method::DELETE
                && path.ends_with("sessions/session-1?api-version=6.0-preview"))
            .count(),
        1
    );
    assert_eq!(
        database
            .store
            .pending_github_jobs(
                target.workspace_id,
                target.organization_id,
                target.scale_set_id
            )
            .unwrap(),
        vec![]
    );
}
