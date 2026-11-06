//! Isolated real PostgreSQL records for hosted lifecycle integration.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use ciproxy_store_interface::gateway::Contact;
use ciproxy_store_interface::labels::{Label, WhenNothingFits};
use ciproxy_store_interface::machines::Machine;
use ciproxy_store_interface::organizations::{AppState, Organization, RepositoryAccess};
use ciproxy_store_interface::own_runners::OwnRunner;
use ciproxy_store_interface::runners::{Runner, RunnerPlace, RunnerState};
use ciproxy_store_interface::scale_sets::ScaleSet;
use ciproxy_store_interface::store::{Store, StoreFactory};
use ciproxy_store_interface::vm_size::VmSize;
use ciproxy_store_interface::workspaces::Workspace;
use ciproxy_store_pg::factory::PgStoreFactory;
use diesel::prelude::*;
use diesel::sql_types;
use reqwest::Url;
use uuid::Uuid;

pub(super) fn at() -> DateTime<Utc> {
    "2026-11-05T12:00:00Z".parse().unwrap()
}

pub(super) struct Fixture {
    pub(super) store: Arc<dyn Store>,
    pub(super) workspace: Uuid,
    pub(super) organization: Uuid,
    pub(super) machine: Uuid,
    pub(super) intent: OwnRunner,
    pub(super) contact: Contact,
    connection: PgConnection,
    admin: PgConnection,
    name: String,
}

impl Fixture {
    pub(super) fn new() -> Self {
        let raw = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL is required for runtime integration tests");
        let mut admin = PgConnection::establish(&raw).unwrap();
        let name = format!("ciproxy_own_{}", Uuid::new_v4().simple());
        diesel::sql_query(format!("CREATE DATABASE {name} TEMPLATE template0"))
            .execute(&mut admin)
            .unwrap();
        let mut url = Url::parse(&raw).unwrap();
        url.set_path(&name);
        let store = PgStoreFactory.connect(url.as_str()).unwrap();
        let workspace = Uuid::new_v4();
        let organization = Uuid::new_v4();
        let machine = Uuid::new_v4();
        store
            .create_workspace(&Workspace {
                id: workspace,
                name: "Own runtime".to_owned(),
                delete_after: None,
            })
            .unwrap();
        store
            .install_organization(&Organization {
                id: organization,
                workspace_id: workspace,
                github_id: 42,
                login: "own-runtime".to_owned(),
                installation_id: 43,
                app_state: AppState::Active,
                repositories: RepositoryAccess::AllPrivate,
                removed_at: None,
                delete_after: None,
            })
            .unwrap();
        let label = Label {
            id: Uuid::new_v4(),
            name: "ciproxy".to_owned(),
            size: VmSize {
                vcpus: 2,
                memory_gb: 8,
                disk_gb: 80,
            },
            allow_machines: true,
            allow_boat: false,
            when_nothing_fits: WhenNothingFits::Wait,
        };
        store.create_label(workspace, &label).unwrap();
        let scale_set = ScaleSet {
            id: Uuid::new_v4(),
            organization_id: organization,
            label_id: label.id,
            github_id: 44,
            runner_group_id: 45,
        };
        store.create_scale_set(workspace, &scale_set).unwrap();
        store
            .create_machine(
                workspace,
                &Machine {
                    id: machine,
                    hostname: "runtime-host".to_owned(),
                    processor: "x64".to_owned(),
                    hardware_threads: 8,
                    memory_bytes: 32_000_000_000,
                    kvm_available: true,
                    slots: 3,
                    warm_runners: 0,
                    draining: true,
                    last_contact_at: Some(at()),
                    agent_version: "0.1.0".to_owned(),
                    disks: vec![],
                    removed_at: None,
                },
            )
            .unwrap();
        let runner_id = Uuid::new_v4();
        let report = serde_json::from_value(serde_json::json!({
            "hostname":"runtime-host", "agent_version":"0.1.0",
            "hardware":{"processor":"x64","threads":8,"memory_bytes":32_000_000_000u64,"kvm":true,"disks":[{"name":"/","capacity_bytes":1_000_000_000_000u64,"free_bytes":800_000_000_000u64}]},
            "capacity":3,"job_disk_bytes":645_000_000_000u64,"slots":[],
            "runners":[{"slot_indices":[0],"runner_id":runner_id,"organization_id":"42","label":"ciproxy","configuration_lost":false,"requested_size":{"vcpu":2,"memory_bytes":8_000_000_000u64,"disk_bytes":80_000_000_000u64},"size":{"vcpu":2,"memory_bytes":8_000_000_000u64,"disk_bytes":142_755_282_944u64},"image_id":"fixture","state":{"state":"warm"}}],
            "drain":"draining","cache":{"used_bytes":0,"capacity_bytes":0,"restores_last_7_days":0,"hits_last_7_days":0,"jobs":[]},"docker_disks":{"leased_bytes":0,"repositories":0,"used_bytes":0},"images":{"available":[{"image_id":"fixture","root_disk_bytes":60_263_759_872u64}],"newest":"fixture"}
        })).unwrap();
        let contact = Contact {
            session_id: Uuid::new_v4(),
            report,
            at: at(),
        };
        let mut connection = PgConnection::establish(url.as_str()).unwrap();
        diesel::sql_query("INSERT INTO machine_reports (workspace_id, machine_id, session_id, report, received_at) VALUES ($1,$2,$3,$4,$5)")
            .bind::<sql_types::Uuid,_>(workspace).bind::<sql_types::Uuid,_>(machine).bind::<sql_types::Uuid,_>(contact.session_id)
            .bind::<sql_types::Jsonb,_>(serde_json::to_value(&contact.report).unwrap()).bind::<sql_types::Timestamptz,_>(contact.at).execute(&mut connection).unwrap();
        let intent = OwnRunner {
            requested_size: VmSize {
                vcpus: 2,
                memory_gb: 8,
                disk_gb: 80,
            },
            slot_indices: vec![0],
            runner: Runner {
                id: runner_id,
                organization_id: organization,
                label_id: label.id,
                scale_set_id: scale_set.id,
                github_id: Some(47),
                state: RunnerState::Ready,
                size: ciproxy_protocol::resources::VmSize {
                    vcpu: 2,
                    memory_bytes: 8_000_000_000,
                    disk_bytes: 142_755_282_944,
                },
                place: RunnerPlace::Machine {
                    machine_id: machine,
                    first_slot: 0,
                },
            },
            label_name: "ciproxy".to_owned(),
            command_id: Uuid::new_v4(),
            issued_at: at() - chrono::Duration::minutes(1),
            acknowledged: true,
            confirmed_at: None,
        };
        store.create_own_runner(workspace, &intent).unwrap();
        Self {
            store,
            workspace,
            organization,
            machine,
            intent,
            contact,
            admin,
            connection,
            name,
        }
    }
    pub(super) fn add_machine(&mut self) -> Uuid {
        let source = self
            .store
            .machine_snapshots(self.workspace)
            .unwrap()
            .remove(0);
        let mut machine = source.machine;
        machine.id = Uuid::new_v4();
        machine.draining = false;
        machine.warm_runners = 1;
        self.store.create_machine(self.workspace, &machine).unwrap();
        let mut report = source.report.unwrap().report;
        report.runners.clear();
        diesel::sql_query("INSERT INTO machine_reports (workspace_id, machine_id, session_id, report, received_at) VALUES ($1,$2,$3,$4,$5)")
            .bind::<sql_types::Uuid,_>(self.workspace).bind::<sql_types::Uuid,_>(machine.id).bind::<sql_types::Uuid,_>(Uuid::new_v4())
            .bind::<sql_types::Jsonb,_>(serde_json::to_value(report).unwrap()).bind::<sql_types::Timestamptz,_>(at()).execute(&mut self.connection).unwrap();
        machine.id
    }

    pub(super) fn hold_organization(&mut self) {
        diesel::sql_query("UPDATE organization_activation SET admitted=FALSE,admin_approval_hold=TRUE WHERE workspace_id=$1 AND organization_id=$2")
            .bind::<sql_types::Uuid,_>(self.workspace).bind::<sql_types::Uuid,_>(self.organization)
            .execute(&mut self.connection).unwrap();
    }

    pub(super) fn admit_organization(&mut self) {
        diesel::sql_query("UPDATE organization_activation SET admitted=TRUE,admin_approval_hold=FALSE WHERE workspace_id=$1 AND organization_id=$2")
            .bind::<sql_types::Uuid,_>(self.workspace).bind::<sql_types::Uuid,_>(self.organization)
            .execute(&mut self.connection).unwrap();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        diesel::sql_query(format!("DROP DATABASE {} WITH (FORCE)", self.name))
            .execute(&mut self.admin)
            .unwrap();
    }
}
