//! Settings, labels, repository isolation, and intentional-removal recovery.

use std::sync::atomic::Ordering;

use axum::http::StatusCode;
use ciproxy_api::maintenance::{ApiMaintenance, Maintenance};
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::github::{
    DemandPhase, GithubJob, Repository, RepositoryIdentity, RunFacts,
};
use ciproxy_store_interface::jobs::{Job, JobState};
use ciproxy_store_interface::organizations::{AppState, RepositoryAccess};
use serde_json::json;
use uuid::Uuid;

use super::fixture::Fixture;
use super::fixture_io::json;

#[tokio::test]
async fn repositories_remain_per_organization_and_takeout_recovers_without_retention() {
    let fixture = Fixture::new();
    let admin = fixture.login(7, Some(Role::Admin)).await;
    let last = fixture.path(&format!("/organizations/{}", fixture.organization.id));
    assert_eq!(
        fixture
            .request(Some(&admin), "DELETE", &last, None)
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
    let mut second = fixture.organization.clone();
    second.id = Uuid::new_v4();
    second.github_id = 200;
    second.installation_id = 201;
    second.login = "other".to_owned();
    fixture
        .database
        .store
        .install_organization(&second)
        .unwrap();
    fixture
        .remote
        .organizations
        .lock()
        .unwrap()
        .push(second.clone());
    fixture.identity.memberships.lock().unwrap().insert(
        7,
        vec![
            ciproxy_store_interface::dashboard::records::Membership {
                github_id: 100,
                owner: false,
            },
            ciproxy_store_interface::dashboard::records::Membership {
                github_id: 200,
                owner: false,
            },
        ],
    );
    fixture.advance(chrono::Duration::hours(1));
    let _removed_member = fixture.login(8, Some(Role::Viewer)).await;
    fixture
        .database
        .store
        .replace_repositories(
            fixture.workspace,
            fixture.organization.id,
            &[
                Repository {
                    id: 301,
                    full_name: "customer/private".to_owned(),
                    private: true,
                    default_branch: "main".to_owned(),
                },
                Repository {
                    id: 302,
                    full_name: "customer/public".to_owned(),
                    private: false,
                    default_branch: "main".to_owned(),
                },
            ],
        )
        .unwrap();
    let repositories = fixture.path(&format!(
        "/organizations/{}/repositories",
        fixture.organization.id
    ));
    let list = json(
        fixture
            .request(Some(&admin), "GET", &repositories, None)
            .await,
    )
    .await;
    assert_eq!(list[0]["allowed"], true);
    assert_eq!(list[1]["allowed"], false);
    assert_eq!(
        fixture
            .request(
                Some(&admin),
                "PUT",
                &repositories,
                Some(json!({"mode":"selected","ids":["301"]}))
            )
            .await
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        fixture
            .database
            .store
            .organization(fixture.workspace, second.id)
            .unwrap()
            .unwrap()
            .repositories,
        RepositoryAccess::AllPrivate
    );
    let sets = fixture
        .database
        .store
        .scale_sets(fixture.workspace, fixture.organization.id)
        .unwrap();
    let job = Job {
        id: Uuid::new_v4(),
        organization_id: fixture.organization.id,
        github_id: 900,
        label_id: sets[0].label_id,
        scale_set_id: sets[0].id,
        runner_id: None,
        name: "Build".to_owned(),
        workflow: "CI".to_owned(),
        repository: "customer/private".to_owned(),
        git_ref: None,
        state: JobState::Running,
        result: None,
        queued_at: "2026-11-06T11:00:00Z".parse().unwrap(),
        started_at: Some("2026-11-06T11:00:05Z".parse().unwrap()),
        completed_at: None,
        routing_reason: None,
        cache_source: None,
        machine_handled_cache_counts: None,
    };
    fixture
        .database
        .store
        .insert_job(fixture.workspace, &job)
        .unwrap();
    let demand = GithubJob {
        id: Uuid::new_v4(),
        organization_id: fixture.organization.id,
        scale_set_id: sets[0].id,
        label_id: sets[0].label_id,
        actions_job_id: None,
        request_id: 0,
        run_id: 9000,
        repository: "customer/private".to_owned(),
        name: "Build".to_owned(),
        workflow_ref: "customer/private/.github/workflows/ci.yml@main".to_owned(),
        phase: DemandPhase::Started,
        result: None,
        assigned_at: None,
        started_at: Some("2026-11-06T11:00:05Z".parse().unwrap()),
        completed_at: None,
        runner_github_id: None,
        facts: RunFacts {
            id: 9000,
            repository: RepositoryIdentity {
                id: 301,
                full_name: "customer/private".to_owned(),
            },
            event: "push".to_owned(),
            head_branch: Some("main".to_owned()),
            head_sha: "trusted-sha".to_owned(),
            head_repository: Some(RepositoryIdentity {
                id: 301,
                full_name: "customer/private".to_owned(),
            }),
        },
    };
    fixture
        .database
        .store
        .save_github_job(
            fixture.workspace,
            &demand,
            "2026-11-06T12:00:00Z".parse().unwrap(),
        )
        .unwrap();
    let setup = json(
        fixture
            .request(Some(&admin), "GET", &fixture.path("/setup"), None)
            .await,
    )
    .await;
    assert_eq!(setup["first_job"]["name"], "Build");
    assert_eq!(setup["first_job"]["phase"], "started");
    assert_eq!(setup["first_job"]["id"], demand.id.to_string());
    assert!(setup["first_job"]["assigned_at"].is_null());
    assert!(setup["first_job"]["completed_at"].is_null());
    assert!(setup["first_job"]["place"].is_null());
    fixture.remote.fail_uninstall.store(true, Ordering::SeqCst);
    assert_eq!(
        fixture
            .request(Some(&admin), "DELETE", &last, None)
            .await
            .status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    let removed = fixture
        .database
        .store
        .organization(fixture.workspace, fixture.organization.id)
        .unwrap()
        .unwrap();
    assert_eq!(removed.app_state, AppState::Removed);
    assert!(removed.delete_after.is_none());
    assert!(
        fixture
            .database
            .store
            .dashboard_user_by_github(8)
            .unwrap()
            .is_none()
    );
    let maintenance = Maintenance {
        state: fixture.state.clone(),
    };
    maintenance.reconcile().await.unwrap();
    assert!(
        fixture
            .database
            .store
            .pending_organization_takeouts()
            .unwrap()
            .is_empty()
    );
    let events = fixture.remote.events.lock().unwrap().clone();
    let uninstall = events
        .iter()
        .position(|event| event == "uninstall:101")
        .unwrap();
    assert!(
        events[..uninstall]
            .iter()
            .any(|event| event.starts_with("delete:"))
    );
    assert!(
        events[..uninstall]
            .iter()
            .any(|event| event == "lifecycle:101")
    );
    assert_eq!(
        fixture
            .database
            .store
            .jobs(fixture.workspace, fixture.organization.id)
            .unwrap()
            .len(),
        1
    );
    fixture
        .database
        .store
        .remove_github_installation(
            fixture.workspace,
            fixture.organization.id,
            101,
            "2026-11-06T12:30:00Z".parse().unwrap(),
        )
        .unwrap();
    assert!(
        fixture
            .database
            .store
            .organization(fixture.workspace, fixture.organization.id)
            .unwrap()
            .unwrap()
            .delete_after
            .is_none()
    );
    let list = json(
        fixture
            .request(Some(&admin), "GET", &fixture.path("/organizations"), None)
            .await,
    )
    .await;
    assert_eq!(list.as_array().unwrap().len(), 2);
}
