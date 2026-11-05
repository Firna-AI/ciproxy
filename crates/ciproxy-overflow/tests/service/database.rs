//! Unique PostgreSQL databases and fixtures created through the public store.

use std::env;
use std::sync::Arc;

use ciproxy_store_interface::labels::{Label, WhenNothingFits};
use ciproxy_store_interface::organizations::{AppState, Organization, RepositoryAccess};
use ciproxy_store_interface::overflow::store::OverflowSession;
use ciproxy_store_interface::scale_sets::ScaleSet;
use ciproxy_store_interface::store::{Store, StoreFactory};
use ciproxy_store_interface::vm_size::VmSize;
use ciproxy_store_interface::workspaces::Workspace;
use ciproxy_store_pg::factory::PgStoreFactory;
use diesel::prelude::*;
use url::Url;

use super::fixtures::id;

pub(super) struct Database {
    pub(super) store: Arc<dyn Store>,
    maintenance: PgConnection,
    name: String,
}

impl Database {
    pub(super) fn new() -> Self {
        let raw_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL is required for overflow PostgreSQL integration tests");
        let mut maintenance = PgConnection::establish(&raw_url).unwrap();
        let name = format!("ciproxy_overflow_{}", uuid::Uuid::new_v4().simple());
        diesel::sql_query(format!("CREATE DATABASE {name} TEMPLATE template0"))
            .execute(&mut maintenance)
            .unwrap();
        let mut url = Url::parse(&raw_url).unwrap();
        url.set_path(&name);
        let database = Self {
            store: PgStoreFactory.connect(url.as_str()).unwrap(),
            maintenance,
            name,
        };
        database.tenant(1);
        database
    }

    pub(super) fn tenant(&self, workspace: u128) {
        self.store
            .create_workspace(&Workspace {
                id: id(workspace),
                name: "Overflow integration workspace".to_owned(),
                delete_after: None,
            })
            .unwrap();
        self.store
            .create_label(
                id(workspace),
                &Label {
                    id: id(workspace + 3),
                    name: "ciproxy".to_owned(),
                    size: VmSize {
                        vcpus: 2,
                        memory_gb: 8,
                        disk_gb: 60,
                    },
                    allow_machines: true,
                    allow_boat: true,
                    when_nothing_fits: WhenNothingFits::Wait,
                },
            )
            .unwrap();
        self.organization_in(workspace, workspace + 2, workspace + 3, workspace + 4);
    }

    pub(super) fn organization(&self, organization: u128, scale_set: u128) {
        self.organization_in(1, organization, 4, scale_set);
    }

    fn organization_in(&self, workspace: u128, organization: u128, label: u128, scale_set: u128) {
        self.store
            .install_organization(&Organization {
                id: id(organization),
                workspace_id: id(workspace),
                repositories: RepositoryAccess::AllPrivate,
                github_id: i64::try_from(organization).unwrap(),
                login: format!("org-{organization}"),
                installation_id: i64::try_from(organization).unwrap(),
                app_state: AppState::Active,
                removed_at: None,
                delete_after: None,
            })
            .unwrap();
        self.store
            .create_scale_set(
                id(workspace),
                &ScaleSet {
                    id: id(scale_set),
                    organization_id: id(organization),
                    label_id: id(label),
                    github_id: i64::try_from(scale_set).unwrap(),
                    runner_group_id: 41,
                },
            )
            .unwrap();
    }

    pub(super) fn session(&self) -> Box<dyn OverflowSession> {
        self.store.lock_overflow(id(1)).unwrap()
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        diesel::sql_query(format!("DROP DATABASE {} WITH (FORCE)", self.name))
            .execute(&mut self.maintenance)
            .expect("remove only the isolated overflow database");
    }
}
