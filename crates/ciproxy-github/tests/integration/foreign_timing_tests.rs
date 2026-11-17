//! Aliased listener attempts recover through a retained same-organization runner.

use std::sync::Arc;
use std::sync::atomic::AtomicI64;

use ciproxy_github::auth::tokens::RotatingTokens;
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::rest::http::RestClient;
use ciproxy_github::timing::{JobTiming, RepairTiming};
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::scale_sets::ScaleSet;
use reqwest::{Client, Url};
use serde_json::json;

use super::database::Database;
use super::fake_http::Server;
use super::foreign_timing_support::{registration, runner, started};
use super::support::{FixedJwt, FixedRuntime, id, setup};

#[tokio::test]
async fn aliased_attempt_recovers_with_retained_runner_only_in_same_github_organization() {
    let database = Database::new();
    let store = &*database.store;
    setup(store);
    let original = store.scale_sets(id(1), id(5)).unwrap().remove(0);
    let retained = runner(store, id(1), &original, 71);
    let original_control_runner = runner(store, id(1), &original, 74);
    let original_control = started(store, id(1), &original, 74);
    store
        .record_app_removal(id(1), id(5), "2026-11-04T12:00:00Z".parse().unwrap())
        .unwrap();
    let alias = registration(store, 90, 17);
    let foreign = started(store, id(90), &alias, 71);
    let replacement = ScaleSet {
        github_id: 83,
        ..alias
    };
    store.save_github_scale_set(id(90), &replacement).unwrap();
    let current_runner = runner(store, id(90), &replacement, 72);
    let current_control = started(store, id(90), &replacement, 72);
    let unrelated = registration(store, 190, 99);
    let unrelated_runner = runner(store, id(190), &unrelated, 73);
    let unrelated_control = started(store, id(90), &replacement, 73);
    let pending = store.incomplete_github_job_attempts(id(90)).unwrap();
    assert_eq!(
        pending
            .iter()
            .map(|attempt| attempt.job.id)
            .collect::<Vec<_>>(),
        vec![foreign.id],
        "only the recorded runner from the same GitHub organization proves a lost completion"
    );
    assert_eq!(pending[0].job, foreign);
    assert!(pending[0].awaiting_completion);
    assert!(
        store
            .incomplete_github_job_attempts(id(1))
            .unwrap()
            .is_empty()
    );
    let server = Server::start(None).await;
    *server.state.job_pages.lock().unwrap() = vec![json!({"total_count":1,"jobs":[{
        "id":333,"run_id":31,"runner_id":71,"started_at":"2026-11-04T12:00:00.000000001Z",
        "completed_at":"2026-11-04T12:00:20.000000002Z","status":"completed","conclusion":"failure"
    }]})];
    let runtime = Arc::new(FixedRuntime(AtomicI64::new(1_793_793_630)));
    let http = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let api = Url::parse(&format!("{}/api/v3/", server.origin)).unwrap();
    let tokens = Arc::new(RotatingTokens::new(
        http.clone(),
        api.clone(),
        Arc::new(FixedJwt),
        runtime.clone(),
    ));
    let repair = RepairTiming::new(
        Arc::new(StoreAdapter::new(database.store.clone(), runtime.clone())),
        Arc::new(RestClient::new(http, api, tokens)),
        runtime,
    );
    repair.repair(id(90)).await.unwrap();
    let mut completed = foreign;
    completed.phase = DemandPhase::Completed;
    completed.actions_job_id = Some(333);
    completed.completed_at = Some("2026-11-04T12:00:20.000000002Z".parse().unwrap());
    completed.result = Some("failure".to_owned());
    assert_eq!(
        store.github_jobs(id(90), id(93)).unwrap(),
        vec![completed, current_control, unrelated_control]
    );
    assert_eq!(
        store.github_jobs(id(1), id(5)).unwrap(),
        vec![original_control]
    );
    assert_eq!(
        store.runners(id(1), id(5)).unwrap(),
        vec![retained, original_control_runner]
    );
    assert_eq!(store.runners(id(90), id(93)).unwrap(), vec![current_runner]);
    assert_eq!(
        store.runners(id(190), id(193)).unwrap(),
        vec![unrelated_runner]
    );
    assert!(
        store
            .incomplete_github_job_attempts(id(90))
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        server
            .state
            .requests
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, path, _)| path.contains("/jobs?"))
            .count(),
        1
    );
}
