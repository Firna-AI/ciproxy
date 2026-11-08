//! Additional owned registration identities for retention boundaries.

use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::boat_runtime::BoatRunnerIntent;
use ciproxy_store_interface::workspaces::Workspace;
use uuid::Uuid;

use super::overflow_support::Harness;

/// Save a registration in a sibling organization or a separate workspace.
pub(super) async fn extra_intent(
    harness: &Harness,
    source: &BoatRunnerIntent,
    separate_workspace: bool,
) -> (Uuid, BoatRunnerIntent) {
    let fixture = &harness.fixture;
    let store = &fixture.store;
    let workspace = if separate_workspace {
        let id = Uuid::new_v4();
        store
            .create_workspace(&Workspace {
                id,
                name: "Other workspace".to_owned(),
                delete_after: None,
            })
            .unwrap();
        id
    } else {
        fixture.workspace
    };
    let mut organization = store
        .organization(fixture.workspace, fixture.organization)
        .unwrap()
        .unwrap();
    organization.id = Uuid::new_v4();
    organization.workspace_id = workspace;
    organization.github_id = if separate_workspace { 94 } else { 92 };
    organization.installation_id = organization.github_id + 1;
    store.install_organization(&organization).unwrap();
    if separate_workspace {
        harness
            .service
            .connect(
                workspace,
                Uuid::new_v4(),
                Secret::new("fake-other-boat-key"),
                "personal".to_owned(),
                200_000_000,
            )
            .await
            .unwrap();
    }
    let mut label = store.labels(fixture.workspace).unwrap().remove(0);
    if separate_workspace {
        label.id = Uuid::new_v4();
        store.create_label(workspace, &label).unwrap();
    }
    let mut scale_set = store
        .scale_sets(fixture.workspace, fixture.organization)
        .unwrap()
        .remove(0);
    scale_set.id = Uuid::new_v4();
    scale_set.organization_id = organization.id;
    scale_set.label_id = label.id;
    store.create_scale_set(workspace, &scale_set).unwrap();
    let mut intent = source.clone();
    intent.runner_id = Uuid::new_v4();
    intent.delivery_id = Uuid::new_v4();
    intent.organization_id = organization.id;
    intent.label_id = label.id;
    intent.scale_set_id = scale_set.id;
    store.create_boat_runner_intent(workspace, &intent).unwrap();
    (workspace, intent)
}

/// Send the saved guest end token through the production overflow entry point.
pub(super) async fn report_end(harness: &Harness, runner: Uuid) {
    let token = harness
        .ports
        .inputs
        .lock()
        .unwrap()
        .iter()
        .find(|input| input["configure"]["runner_id"] == runner.to_string())
        .unwrap()["exit_report_token"]
        .as_str()
        .unwrap()
        .to_owned();
    harness
        .service
        .report_end(Secret::new(token), runner)
        .await
        .unwrap();
}
