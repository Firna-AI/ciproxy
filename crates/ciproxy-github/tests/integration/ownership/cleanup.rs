//! A same-workspace return after every foreign binding completed cleanup.

use std::sync::Arc;

use ciproxy_github::app::webhook::InstallationAction;
use ciproxy_github::app::{AppService, GitHubApp};
use ciproxy_github::scale_sets::ScaleSetFactory;
use ciproxy_store_interface::organizations::AppState;
use ciproxy_store_interface::runners::{Runner, RunnerState};
use ciproxy_store_interface::store::Store;

use super::super::app_fixture::{Fixture, at, installed};
use super::super::resource_fixture::seed_installation_choice;
use super::super::support::{Factory, id};
use super::fixture::event;
use super::http::Server;

pub(super) async fn return_after_takeout_cleanup(
    store: Arc<dyn Store>,
    app: &AppService,
    fixture: &Fixture,
    server: &Server,
    retained_idle: &Runner,
) {
    let occupied_name = "ciproxy-retired-00000000000000000000000000000006";
    let retired_name = "ciproxy-retired-00000000000000000000000000000006-1";
    server.state.lock().unwrap().sets.get_mut(&54).unwrap().name = occupied_name.to_owned();
    let mut label = store.labels(id(90)).unwrap().remove(0);
    label.id = id(99);
    label.name = retired_name.to_owned();
    store.create_label(id(90), &label).unwrap();
    app.installation(event(InstallationAction::Deleted, 8))
        .await
        .unwrap();
    *fixture.state.current.lock().unwrap() = Some(installed(9, false));
    seed_installation_choice(store.as_ref(), &installed(9, false), id(90), 41);
    app.installation(event(InstallationAction::Created, 9))
        .await
        .unwrap();
    assert_eq!(server.eligible_runners(retired_name), Vec::<i64>::new());
    assert_ne!(server.state.lock().unwrap().sets[&53].name, retired_name);
    let current = store
        .organizations_by_github_id(17)
        .unwrap()
        .into_iter()
        .find(|organization| organization.app_state == AppState::Active)
        .unwrap();
    let takeout = store
        .begin_organization_takeout(id(90), current.id)
        .unwrap();
    let api = Factory(server.origin.clone())
        .for_organization(&current)
        .await
        .unwrap();
    api.remove_runner(retained_idle.github_id.unwrap())
        .await
        .unwrap();
    store
        .update_runner(
            id(90),
            retained_idle.organization_id,
            retained_idle.id,
            RunnerState::Stopped,
            retained_idle.github_id,
        )
        .unwrap();
    for registration in store
        .organizations_by_github_id(17)
        .unwrap()
        .into_iter()
        .filter(|organization| organization.workspace_id == id(90))
    {
        for set in store.dashboard_scale_sets(id(90), registration.id).unwrap() {
            api.remove_scale_set(set.github_id).await.unwrap();
            store
                .complete_scale_set_removal(id(90), &set, at())
                .unwrap();
        }
        assert_eq!(
            store.dashboard_scale_sets(id(90), registration.id).unwrap(),
            vec![]
        );
    }
    store.complete_organization_takeout(&takeout).unwrap();
    *fixture.state.current.lock().unwrap() = Some(installed(10, false));
    seed_installation_choice(store.as_ref(), &installed(10, false), id(1), 41);
    app.installation(event(InstallationAction::Created, 10))
        .await
        .unwrap();
}
