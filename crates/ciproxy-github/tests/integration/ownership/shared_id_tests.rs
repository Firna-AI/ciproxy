//! Legacy shared remote IDs cannot give foreign runners active eligibility.

use ciproxy_github::app::GitHubApp;
use ciproxy_github::app::webhook::InstallationAction;
use ciproxy_store_interface::organizations::{AppState, Organization, RepositoryAccess};
use ciproxy_store_interface::runners::RunnerState;
use ciproxy_store_interface::scale_sets::{RegistrationTarget, ScaleSet};

use super::super::app_fixture::{Fixture, at, installed};
use super::super::database::Database;
use super::super::resource_fixture::seed_installation_choice;
use super::super::support::{id, setup};
use super::fixture::{app, event, register_runner, target_workspace};
use super::http::Server;

#[tokio::test]
async fn legacy_shared_id_gets_fresh_binding_and_keeps_captured_registration_targets() {
    let database = Database::new();
    setup(&*database.store);
    let workspace = target_workspace(&*database.store);
    let original_set = database.store.scale_sets(id(1), id(5)).unwrap().remove(0);
    let server = Server::start().await;
    let original_idle = register_runner(
        database.store.clone(),
        &server.origin,
        id(1),
        &original_set,
        401,
        RunnerState::Ready,
    )
    .await;
    database
        .store
        .record_app_removal(id(1), id(5), at())
        .unwrap();
    let organization = Organization {
        id: id(93),
        workspace_id: workspace,
        github_id: 17,
        login: "customer".to_owned(),
        installation_id: 7,
        app_state: AppState::Active,
        repositories: RepositoryAccess::AllPrivate,
        removed_at: None,
        delete_after: None,
    };
    database.store.install_organization(&organization).unwrap();
    let shared_set = ScaleSet {
        id: id(94),
        organization_id: organization.id,
        label_id: id(91),
        github_id: 53,
        runner_group_id: 47,
    };
    database
        .store
        .create_scale_set(workspace, &shared_set)
        .unwrap();
    let shared_idle = register_runner(
        database.store.clone(),
        &server.origin,
        workspace,
        &shared_set,
        402,
        RunnerState::Ready,
    )
    .await;
    assert_eq!(server.eligible_runners("ciproxy"), vec![71, 72]);
    let fixture = Fixture::new();
    *fixture.state.current.lock().unwrap() = Some(installed(7, false));
    let app = app(database.store.clone(), &fixture, &server.origin);
    app.refresh_repositories(7).await.unwrap();
    let saved = database
        .store
        .scale_sets(workspace, organization.id)
        .unwrap();
    let base = saved.iter().find(|set| set.label_id == id(91)).unwrap();
    assert_eq!(base.id, shared_set.id);
    assert_eq!(
        base.github_id, 83,
        "a saved ID shared with another workspace cannot be reused"
    );
    assert_eq!(server.eligible_runners("ciproxy"), Vec::<i64>::new());
    assert_eq!(
        database.store.scale_sets(id(1), id(5)).unwrap(),
        vec![original_set.clone()]
    );
    assert_eq!(
        database.store.runners(id(1), id(5)).unwrap(),
        vec![original_idle.clone()]
    );
    assert_eq!(
        database.store.runners(workspace, organization.id).unwrap(),
        vec![shared_idle.clone()]
    );
    assert_eq!(
        database
            .store
            .runner_registration_target(id(1), id(5), original_idle.id)
            .unwrap(),
        Some(RegistrationTarget::from(&original_set))
    );
    assert_eq!(
        database
            .store
            .runner_registration_target(workspace, organization.id, shared_idle.id)
            .unwrap(),
        Some(RegistrationTarget::from(&shared_set))
    );
    app.refresh_repositories(7).await.unwrap();
    assert_eq!(
        database
            .store
            .scale_sets(workspace, organization.id)
            .unwrap(),
        saved
    );
    assert_eq!(server.state.lock().unwrap().creates, vec![83, 84]);
    assert_eq!(server.state.lock().unwrap().runners.len(), 2);
    assert_eq!(server.state.lock().unwrap().deletes, Vec::<i64>::new());
    database
        .store
        .begin_organization_takeout(workspace, organization.id)
        .unwrap();
    *fixture.state.current.lock().unwrap() = Some(installed(8, false));
    seed_installation_choice(&*database.store, &installed(8, false), id(1), 41);
    app.installation(event(InstallationAction::Created, 8))
        .await
        .unwrap();
    let returned = database.store.scale_sets(id(1), id(5)).unwrap().remove(0);
    assert_eq!(returned.id, original_set.id);
    assert_eq!(
        returned.github_id, 85,
        "return must never reuse the quarantined remote ID"
    );
    assert_eq!(server.eligible_runners("ciproxy"), Vec::<i64>::new());
    assert_eq!(
        server.state.lock().unwrap().sets[&53].name,
        "ciproxy-retired-00000000000000000000000000000006"
    );
    assert_eq!(
        server.state.lock().unwrap().sets[&53].labels,
        vec!["ciproxy-retired-00000000000000000000000000000006-quarantine"]
    );
    assert_eq!(
        database.store.runners(workspace, organization.id).unwrap(),
        vec![shared_idle.clone()]
    );
    assert_eq!(
        database
            .store
            .runner_registration_target(workspace, organization.id, shared_idle.id)
            .unwrap(),
        Some(RegistrationTarget::from(&shared_set))
    );
    register_runner(
        database.store.clone(),
        &server.origin,
        id(1),
        &returned,
        403,
        RunnerState::Ready,
    )
    .await;
    assert_eq!(server.eligible_runners("ciproxy"), vec![73]);
    assert_eq!(server.state.lock().unwrap().runners.len(), 3);
    assert_eq!(server.state.lock().unwrap().deletes, Vec::<i64>::new());
    for (label_id, name) in [
        (404, "ciproxy-retired-00000000000000000000000000000006"),
        (405, "ciproxy-quarantined-53-quarantine"),
    ] {
        let mut label = database.store.labels(id(1)).unwrap().remove(0);
        label.id = id(label_id);
        label.name = name.to_owned();
        database.store.create_label(id(1), &label).unwrap();
        app.refresh_repositories(8).await.unwrap();
        let bindings = database.store.scale_sets(id(1), id(5)).unwrap();
        let binding = bindings
            .iter()
            .find(|set| set.label_id == label.id)
            .unwrap();
        assert_ne!(
            binding.github_id, 53,
            "a valid customer label cannot adopt a quarantine marker"
        );
        assert_eq!(server.eligible_runners(name), Vec::<i64>::new());
        let remote = server.state.lock().unwrap();
        assert_eq!(remote.sets[&binding.github_id].name, name);
        assert_eq!(remote.sets[&binding.github_id].labels, vec![name]);
    }
    app.refresh_repositories(8).await.unwrap();
    assert_eq!(
        server.state.lock().unwrap().creates,
        vec![83, 84, 85, 86, 87],
        "valid custom labels with an internal prefix must remain reusable"
    );
    assert_eq!(server.eligible_runners("ciproxy"), vec![73]);
    assert_eq!(server.state.lock().unwrap().runners.len(), 3);
    assert_eq!(server.state.lock().unwrap().deletes, Vec::<i64>::new());
}
