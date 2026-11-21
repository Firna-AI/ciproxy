//! Workspace changes preserve retained runners without sharing active labels.

use ciproxy_github::app::GitHubApp;
use ciproxy_github::app::webhook::InstallationAction;
use ciproxy_github::error::Error;
use ciproxy_store_interface::organizations::AppState;
use ciproxy_store_interface::runners::RunnerState;
use ciproxy_store_interface::scale_sets::{RegistrationTarget, ScaleSet};

use super::super::app_fixture::{Fixture, at, installed};
use super::super::database::Database;
use super::super::resource_fixture::seed_installation_choice;
use super::super::support::{id, setup};
use super::cleanup::return_after_takeout_cleanup;
use super::fixture::{app, event, register_runner, running_job, target_workspace};
use super::http::{RemoteSet, Server};

#[tokio::test]
async fn transfer_retries_and_return_preserve_running_and_idle_registration_ownership() {
    let database = Database::new();
    let original = setup(&*database.store);
    let workspace = target_workspace(&*database.store);
    let original_set = database.store.scale_sets(id(1), id(5)).unwrap().remove(0);
    let mut running_label = database.store.labels(id(1)).unwrap().remove(0);
    running_label.id = id(3);
    running_label.name = "ciproxy-running".to_owned();
    database.store.create_label(id(1), &running_label).unwrap();
    let running_set = ScaleSet {
        id: id(7),
        label_id: id(3),
        github_id: 54,
        ..original_set.clone()
    };
    database
        .store
        .create_scale_set(id(1), &running_set)
        .unwrap();
    let server = Server::start().await;
    server.state.lock().unwrap().sets.insert(
        54,
        RemoteSet {
            id: 54,
            name: "ciproxy-running".to_owned(),
            group: 47,
            labels: vec!["ciproxy-running".to_owned()],
        },
    );
    let idle = register_runner(
        database.store.clone(),
        &server.origin,
        id(1),
        &original_set,
        401,
        RunnerState::Ready,
    )
    .await;
    let busy = register_runner(
        database.store.clone(),
        &server.origin,
        id(1),
        &running_set,
        402,
        RunnerState::Busy,
    )
    .await;
    let job = running_job(&running_set, &busy);
    database.store.save_github_job(id(1), &job, at()).unwrap();
    let retained_runners = database.store.runners(id(1), id(5)).unwrap();
    let pending = database
        .store
        .pending_github_jobs(id(1), id(5), running_set.id)
        .unwrap();
    let fixture = Fixture::new();
    let app = app(database.store.clone(), &fixture, &server.origin);
    app.installation(event(InstallationAction::Deleted, 19))
        .await
        .unwrap();
    *fixture.state.current.lock().unwrap() = Some(installed(7, false));
    seed_installation_choice(&*database.store, &installed(7, false), workspace, 41);
    server.state.lock().unwrap().fail_create = Some("ciproxy-extra".to_owned());
    assert!(matches!(
        app.installation(event(InstallationAction::Created, 7))
            .await,
        Err(Error::Internal(_))
    ));
    assert_eq!(
        server.eligible_runners("ciproxy"),
        Vec::<i64>::new(),
        "the retained idle registration must lose the active label before activation"
    );
    assert_eq!(
        server.eligible_runners("ciproxy-running"),
        Vec::<i64>::new()
    );
    assert_eq!(
        database
            .store
            .organizations_by_github_id(17)
            .unwrap()
            .iter()
            .filter(|organization| organization.app_state == AppState::Active)
            .count(),
        0
    );
    app.installation(event(InstallationAction::Created, 7))
        .await
        .unwrap();
    let current = database
        .store
        .organizations_by_github_id(17)
        .unwrap()
        .into_iter()
        .find(|organization| organization.app_state == AppState::Active)
        .unwrap();
    assert_eq!(current.workspace_id, workspace);
    let current_sets = database.store.scale_sets(workspace, current.id).unwrap();
    let current_set = current_sets
        .iter()
        .find(|set| set.label_id == id(91))
        .unwrap();
    assert_eq!(
        current_set.github_id, 83,
        "the new workspace must not adopt the retained workspace's scale set"
    );
    assert_eq!(
        current_sets
            .iter()
            .map(|set| set.github_id)
            .collect::<Vec<_>>(),
        vec![83, 84]
    );
    assert_eq!(
        server.state.lock().unwrap().creates,
        vec![83, 84],
        "the retry must reuse the prepared unsaved set"
    );
    let original_binding = database
        .store
        .scale_set_binding(
            original.workspace_id,
            original.organization_id,
            original.scale_set_id,
        )
        .unwrap()
        .unwrap();
    assert_eq!(original_binding.scale_set, original_set);
    assert!(
        !original_binding.has_tracked_jobs,
        "idle registrations alone must prevent deletion through ownership isolation"
    );
    assert_eq!(
        database.store.runners(id(1), id(5)).unwrap(),
        retained_runners
    );
    assert_eq!(
        database.store.github_jobs(id(1), id(5)).unwrap(),
        vec![job.clone()]
    );
    assert_eq!(
        database
            .store
            .pending_github_jobs(id(1), id(5), running_set.id)
            .unwrap(),
        pending
    );
    assert_eq!(
        database
            .store
            .runner_registration_target(id(1), id(5), idle.id)
            .unwrap(),
        Some(RegistrationTarget::from(&original_set))
    );
    assert_eq!(
        database
            .store
            .runner_registration_target(id(1), id(5), busy.id)
            .unwrap(),
        Some(RegistrationTarget::from(&running_set))
    );
    assert_eq!(
        database
            .store
            .listener_scale_sets(id(1), id(5))
            .unwrap()
            .iter()
            .find(|binding| binding.scale_set.id == running_set.id)
            .unwrap()
            .scale_set,
        running_set
    );
    let target_idle = register_runner(
        database.store.clone(),
        &server.origin,
        workspace,
        current_set,
        403,
        RunnerState::Ready,
    )
    .await;
    assert_eq!(server.eligible_runners("ciproxy"), vec![73]);
    database
        .store
        .begin_organization_takeout(workspace, current.id)
        .unwrap();
    let taken_out = database
        .store
        .organization(workspace, current.id)
        .unwrap()
        .unwrap();
    assert_eq!(
        (
            taken_out.app_state,
            taken_out.removed_at,
            taken_out.delete_after
        ),
        (AppState::Removed, None, None)
    );
    *fixture.state.current.lock().unwrap() = Some(installed(8, false));
    seed_installation_choice(&*database.store, &installed(8, false), id(1), 41);
    app.installation(event(InstallationAction::Created, 8))
        .await
        .unwrap();
    assert_eq!(
        database.store.scale_sets(id(1), id(5)).unwrap(),
        vec![original_set.clone(), running_set.clone()]
    );
    assert_eq!(
        database.store.scale_sets(workspace, current.id).unwrap(),
        current_sets
    );
    assert_eq!(server.eligible_runners("ciproxy"), vec![71]);
    assert_eq!(server.eligible_runners("ciproxy-running"), vec![72]);
    assert_eq!(
        database.store.runners(workspace, current.id).unwrap(),
        vec![target_idle.clone()]
    );
    assert_eq!(
        database
            .store
            .runner_registration_target(workspace, current.id, target_idle.id)
            .unwrap(),
        Some(RegistrationTarget::from(current_set))
    );
    assert_eq!(database.store.github_jobs(id(1), id(5)).unwrap(), vec![job]);
    {
        let remote = server.state.lock().unwrap();
        assert_eq!(remote.sets[&53].name, "ciproxy");
        assert_eq!(remote.sets[&54].labels, vec!["ciproxy-running"]);
        assert_eq!(
            remote.sets[&83].name,
            format!("ciproxy-retired-{}", current_set.id.simple())
        );
        assert_eq!(
            remote.sets[&83].labels,
            vec![format!("ciproxy-retired-{}", current_set.id.simple())]
        );
        assert_eq!(remote.runners.len(), 3);
        assert_eq!(remote.deletes, Vec::<i64>::new());
    }
    return_after_takeout_cleanup(
        database.store.clone(),
        &app,
        &fixture,
        &server,
        &target_idle,
    )
    .await;
    assert_eq!(server.eligible_runners("ciproxy"), vec![71]);
    assert_eq!(server.eligible_runners("ciproxy-running"), vec![72]);
    assert_eq!(
        database.store.scale_sets(id(1), id(5)).unwrap(),
        vec![original_set, running_set]
    );
}
