//! Per-test PostgreSQL databases and public API fixtures.

use std::env;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use ciproxy_store_interface::boat::BoatAccount;
use ciproxy_store_interface::jobs::{Job, JobState};
use ciproxy_store_interface::labels::{Label, WhenNothingFits};
use ciproxy_store_interface::machines::{Machine, MachineDisk};
use ciproxy_store_interface::organizations::{AppState, Organization, RepositoryAccess};
use ciproxy_store_interface::runners::{Runner, RunnerPlace, RunnerState};
use ciproxy_store_interface::scale_sets::ScaleSet;
use ciproxy_store_interface::secrets::TokenHash;
use ciproxy_store_interface::store::{Store, StoreFactory};
use ciproxy_store_interface::vm_size::VmSize;
use ciproxy_store_interface::workspaces::Workspace;
use ciproxy_store_pg::factory::PgStoreFactory;
use diesel::prelude::*;
use url::Url;
use uuid::Uuid;

pub(super) struct Database {
    pub(super) store: Arc<dyn Store>,
    pub(super) connection: PgConnection,
    pub(super) url: String,
    _database: TestDatabase,
}

struct TestDatabase {
    maintenance: PgConnection,
    name: String,
}

impl Database {
    pub(super) fn new() -> Self {
        let raw_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL is required for real PostgreSQL store tests");
        let mut admin = PgConnection::establish(&raw_url).expect("connect test PostgreSQL");
        let name = format!("ciproxy_test_{}", Uuid::new_v4().simple());
        diesel::sql_query(format!("CREATE DATABASE {name} TEMPLATE template0"))
            .execute(&mut admin)
            .expect("create isolated test database");
        let database = TestDatabase {
            maintenance: admin,
            name,
        };
        let mut url = Url::parse(&raw_url).expect("DATABASE_URL must be a PostgreSQL URL");
        url.set_path(&database.name);
        let url = url.to_string();
        let store: Arc<dyn Store> = PgStoreFactory.connect(&url).expect("open migrated store");
        let connection = PgConnection::establish(&url).expect("connect test database");
        Self {
            store,
            connection,
            url,
            _database: database,
        }
    }

    pub(super) fn workspace(&self) -> Workspace {
        let id = Uuid::new_v4();
        let record = Workspace {
            id,
            name: format!("Workspace {id}"),
            delete_after: None,
        };
        self.store.create_workspace(&record).unwrap();
        record
    }

    pub(super) fn organization(&self, workspace: &Workspace) -> Organization {
        let id = Uuid::new_v4();
        let github_id = i64::from_be_bytes(id.as_bytes()[..8].try_into().unwrap()) & i64::MAX;
        let record = Organization {
            id,
            workspace_id: workspace.id,
            repositories: RepositoryAccess::AllPrivate,
            github_id,
            login: format!("org-{id}"),
            installation_id: github_id,
            app_state: AppState::Active,
            removed_at: None,
            delete_after: None,
        };
        self.store.install_organization(&record).unwrap();
        record
    }

    pub(super) fn label_and_scale_set(&self, organization: &Organization) -> (Label, ScaleSet) {
        let label = Label {
            id: Uuid::new_v4(),
            name: format!("ciproxy-{}", Uuid::new_v4()),
            size: VmSize {
                vcpus: 2,
                memory_gb: 8,
                disk_gb: 60,
            },
            allow_machines: true,
            allow_boat: true,
            when_nothing_fits: WhenNothingFits::Wait,
        };
        self.store
            .create_label(organization.workspace_id, &label)
            .unwrap();
        let scale_set = self.scale_set(organization, &label);
        (label, scale_set)
    }

    pub(super) fn scale_set(&self, organization: &Organization, label: &Label) -> ScaleSet {
        let id = Uuid::new_v4();
        let record = ScaleSet {
            id,
            organization_id: organization.id,
            label_id: label.id,
            github_id: i64::from_be_bytes(id.as_bytes()[..8].try_into().unwrap()) & i64::MAX,
            runner_group_id: 41,
        };
        self.store
            .create_scale_set(organization.workspace_id, &record)
            .unwrap();
        record
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        diesel::sql_query(format!("DROP DATABASE {} WITH (FORCE)", self.name))
            .execute(&mut self.maintenance)
            .expect("remove isolated test database");
    }
}

pub(super) fn at() -> DateTime<Utc> {
    "2026-11-04T12:00:00Z".parse().unwrap()
}

pub(super) fn machine() -> Machine {
    Machine {
        id: Uuid::new_v4(),
        hostname: "ci-builder".to_owned(),
        processor: "AMD".to_owned(),
        hardware_threads: 16,
        memory_bytes: 64_000_000_000,
        kvm_available: true,
        slots: 6,
        warm_runners: 2,
        draining: false,
        last_contact_at: Some(at()),
        agent_version: "0.1.0".to_owned(),
        disks: vec![MachineDisk {
            path: "/".to_owned(),
            total_bytes: 500_000_000_000,
            free_bytes: 400_000_000_000,
        }],
        removed_at: None,
    }
}

pub(super) fn account() -> BoatAccount {
    BoatAccount {
        id: Uuid::new_v4(),
        account_name: "Customer Boat".to_owned(),
        connected_at: at(),
        retired_at: None,
        rejected_at: None,
    }
}

pub(super) fn hash() -> TokenHash {
    TokenHash::new(Uuid::new_v4().as_bytes().to_vec())
}

pub(super) fn runner(label: &Label, scale_set: &ScaleSet, place: RunnerPlace) -> Runner {
    Runner {
        organization_id: scale_set.organization_id,
        id: Uuid::new_v4(),
        label_id: label.id,
        scale_set_id: scale_set.id,
        github_id: Some(84),
        state: RunnerState::Busy,
        size: label.size,
        place,
    }
}

pub(super) fn job(label: &Label, scale_set: &ScaleSet) -> Job {
    Job {
        organization_id: scale_set.organization_id,
        id: Uuid::new_v4(),
        github_id: 42,
        label_id: label.id,
        scale_set_id: scale_set.id,
        runner_id: None,
        name: "cargo test".to_owned(),
        workflow: "CI".to_owned(),
        repository: "customer/private".to_owned(),
        git_ref: None,
        state: JobState::Queued,
        result: None,
        queued_at: at(),
        started_at: None,
        completed_at: None,
        routing_reason: None,
        cache_source: None,
        cache_restores: None,
        cache_restored_bytes: None,
    }
}
