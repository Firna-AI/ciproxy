//! Started jobs keep real completion recovery after their remote binding moves.

use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use ciproxy_github::auth::tokens::RotatingTokens;
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::rest::http::RestClient;
use ciproxy_github::timing::{JobTiming, RepairTiming};
use ciproxy_protocol::resources::VmSize;
use ciproxy_store_interface::github::{DemandPhase, GithubJob, RepositoryIdentity, RunFacts};
use ciproxy_store_interface::github_timing::JobTimes;
use ciproxy_store_interface::runners::{Runner, RunnerPlace, RunnerState};
use ciproxy_store_interface::scale_sets::{RegistrationTarget, ScaleSet};
use ciproxy_store_interface::store::Store;
use reqwest::{Client, Url};
use serde_json::json;

use super::database::Database;
use super::fake_http::Server;
use super::support::{FixedJwt, FixedRuntime, id, setup};

fn running(store: &dyn Store, binding: &ScaleSet, runner_id: i64) -> GithubJob {
    let runner = Runner {
        id: id(u128::try_from(runner_id).unwrap()),
        organization_id: binding.organization_id,
        label_id: binding.label_id,
        scale_set_id: binding.id,
        github_id: None,
        state: RunnerState::Busy,
        size: VmSize {
            vcpu: 2,
            memory_bytes: 8_000_000_000,
            disk_bytes: 80_000_000_000,
        },
        place: RunnerPlace::Boat {
            key_id: None,
            sandbox_id: format!("runner-{runner_id}"),
            size_name: "small".to_owned(),
        },
    };
    store.create_runner(id(1), &runner).unwrap();
    store
        .save_runner_registration(
            id(1),
            runner.id,
            RegistrationTarget::from(binding),
            runner_id,
        )
        .unwrap();
    let job = GithubJob {
        id: runner.id,
        organization_id: binding.organization_id,
        scale_set_id: binding.id,
        label_id: binding.label_id,
        actions_job_id: None,
        request_id: 0,
        run_id: 31,
        repository: "customer/private".to_owned(),
        name: "running-before-binding-change".to_owned(),
        workflow_ref: "customer/private/.github/workflows/build.yml@refs/heads/main".to_owned(),
        phase: DemandPhase::Started,
        result: None,
        assigned_at: Some("2026-11-04T11:59:59.000000001Z".parse().unwrap()),
        started_at: Some("2026-11-04T12:00:00.000000001Z".parse().unwrap()),
        completed_at: None,
        runner_github_id: Some(runner_id),
        facts: RunFacts {
            id: 31,
            repository: RepositoryIdentity {
                id: 23,
                full_name: "customer/private".to_owned(),
            },
            event: "push".to_owned(),
            head_branch: Some("main".to_owned()),
            head_sha: "0123456789abcdef".to_owned(),
            head_repository: None,
        },
    };
    store
        .save_github_job(id(1), &job, "2026-11-04T12:00:00Z".parse().unwrap())
        .unwrap();
    job
}

#[tokio::test]
async fn legacy_started_attempt_recovers_real_completion_after_binding_replacement() {
    let database = Database::new();
    setup(&*database.store);
    let binding = database.store.scale_sets(id(1), id(5)).unwrap().remove(0);
    let stranded = running(&*database.store, &binding, 71);
    let replacement = ScaleSet {
        github_id: 83,
        ..binding
    };
    database
        .store
        .save_github_scale_set(id(1), &replacement)
        .unwrap();
    let current = running(&*database.store, &replacement, 72);
    let pending = database
        .store
        .incomplete_github_job_attempts(id(1))
        .unwrap();
    assert_eq!(
        pending
            .iter()
            .map(|attempt| attempt.job.id)
            .collect::<Vec<_>>(),
        vec![stranded.id],
        "only the started attempt on the previous remote ID needs completion recovery"
    );
    let server = Server::start(None).await;
    *server.state.job_pages.lock().unwrap() = vec![json!({"total_count":1,"jobs":[{
        "id":333,"run_id":31,"runner_id":71,"started_at":"2026-11-04T12:00:00.000000001Z",
        "completed_at":null,"status":"in_progress","conclusion":null
    }]})];
    let runtime = Arc::new(FixedRuntime(AtomicI64::new(1_793_793_600)));
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
        runtime.clone(),
    );
    for seconds in [0, 0, 29] {
        runtime.0.store(1_793_793_600 + seconds, Ordering::SeqCst);
        repair.repair(id(1)).await.unwrap();
    }
    assert_eq!(
        server
            .state
            .requests
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, path, _)| path.contains("/jobs?"))
            .count(),
        1,
        "a known start must keep backoff while completion is unavailable"
    );
    let still_running = database
        .store
        .github_jobs(id(1), id(5))
        .unwrap()
        .into_iter()
        .find(|job| job.id == stranded.id)
        .unwrap();
    assert_eq!(still_running.phase, DemandPhase::Started);
    assert_eq!(still_running.completed_at, None);
    assert_eq!(still_running.result, None);
    *server.state.job_pages.lock().unwrap() = vec![json!({"total_count":1,"jobs":[{
        "id":333,"run_id":31,"runner_id":71,"started_at":"2026-11-04T12:00:00.000000001Z",
        "completed_at":"2026-11-04T12:00:20.000000002Z","status":"completed","conclusion":"failure"
    }]})];
    runtime.0.store(1_793_793_630, Ordering::SeqCst);
    repair.repair(id(1)).await.unwrap();
    let jobs = database.store.github_jobs(id(1), id(5)).unwrap();
    let recovered = jobs.iter().find(|job| job.id == stranded.id).unwrap();
    assert_eq!(recovered.phase, DemandPhase::Completed);
    assert_eq!(recovered.actions_job_id, Some(333));
    assert_eq!(recovered.started_at, stranded.started_at);
    assert_eq!(
        recovered.completed_at,
        Some("2026-11-04T12:00:20.000000002Z".parse().unwrap())
    );
    assert_eq!(recovered.result.as_deref(), Some("failure"));
    assert_eq!(jobs.iter().find(|job| job.id == current.id), Some(&current));
    database
        .store
        .save_github_job_times(
            id(1),
            pending[0].attempt_id,
            &JobTimes {
                actions_job_id: 333,
                started_at: None,
                completed_at: None,
                result: None,
                terminal: false,
            },
            "2026-11-04T12:00:31Z".parse().unwrap(),
        )
        .unwrap();
    let retained = database.store.github_jobs(id(1), id(5)).unwrap();
    assert_eq!(
        retained.iter().find(|job| job.id == stranded.id),
        Some(recovered)
    );
    assert!(
        database
            .store
            .incomplete_github_job_attempts(id(1))
            .unwrap()
            .is_empty()
    );
}
