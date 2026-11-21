//! Current and retained organization facts for the M18 membership regression.

use ciproxy_protocol::resources::VmSize;
use ciproxy_store_interface::organizations::Organization;
use ciproxy_store_interface::runners::{Runner, RunnerPlace, RunnerState};
use ciproxy_store_interface::scale_sets::ScaleSet;
use diesel::prelude::*;
use uuid::Uuid;

use super::m18_support::Pages;

pub(super) fn organization(pages: &Pages, github: i64, login: &str) -> Organization {
    let mut value = pages.fixture.organization.clone();
    value.id = Uuid::new_v4();
    value.github_id = github;
    value.installation_id = github + 1;
    value.login = login.into();
    pages
        .fixture
        .database
        .store
        .install_organization(&value)
        .unwrap();
    pages
        .fixture
        .remote
        .organizations
        .lock()
        .unwrap()
        .push(value.clone());
    value
}

pub(super) fn runner(pages: &Pages, organization: &Organization, github: i64) -> Runner {
    let label = pages
        .fixture
        .database
        .store
        .labels(pages.fixture.workspace)
        .unwrap()
        .remove(0);
    let scale = match pages
        .fixture
        .database
        .store
        .scale_sets(pages.fixture.workspace, organization.id)
        .unwrap()
        .into_iter()
        .next()
    {
        Some(scale) => scale,
        None => {
            let scale = ScaleSet {
                id: Uuid::new_v4(),
                organization_id: organization.id,
                label_id: label.id,
                github_id: github + 500,
                runner_group_id: 47,
            };
            pages
                .fixture
                .database
                .store
                .create_scale_set(pages.fixture.workspace, &scale)
                .unwrap();
            scale
        }
    };
    let value = Runner {
        id: Uuid::new_v4(),
        organization_id: organization.id,
        label_id: label.id,
        scale_set_id: scale.id,
        github_id: Some(github),
        state: RunnerState::Busy,
        size: VmSize {
            vcpu: 2,
            memory_bytes: 8_000_000_000,
            disk_bytes: 60_000_000_000,
        },
        place: RunnerPlace::Boat {
            key_id: None,
            sandbox_id: format!("retained-{github}"),
            size_name: "retained-size".into(),
        },
    };
    pages
        .fixture
        .database
        .store
        .create_runner(pages.fixture.workspace, &value)
        .unwrap();
    value
}

pub(super) fn hold(pages: &Pages, organization: &Organization) {
    let mut connection = PgConnection::establish(&pages.fixture.database.url).unwrap();
    diesel::sql_query("UPDATE organization_activation SET admitted=false,admin_approval_hold=true WHERE organization_id=$1")
        .bind::<diesel::sql_types::Uuid, _>(organization.id).execute(&mut connection).unwrap();
}
