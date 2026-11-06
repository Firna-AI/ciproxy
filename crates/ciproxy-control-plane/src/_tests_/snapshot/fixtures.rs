//! Pure routing inputs with explicit trusted times and capacity.

use chrono::{DateTime, Utc};
use ciproxy_store_interface::gateway::{MachineSnapshot, ReceivedReport};
use ciproxy_store_interface::github::{DemandPhase, GithubJob, RepositoryIdentity, RunFacts};
use ciproxy_store_interface::labels::{Label, WhenNothingFits};
use ciproxy_store_interface::machines::Machine;
use ciproxy_store_interface::organizations::{AppState, Organization, RepositoryAccess};
use ciproxy_store_interface::own_runners::RoutingSnapshot;
use ciproxy_store_interface::settings::{OverflowWait, Settings};
use ciproxy_store_interface::vm_size::VmSize;
use uuid::Uuid;

pub(super) fn at() -> DateTime<Utc> {
    "2026-11-05T12:00:00Z".parse().unwrap()
}

pub(super) fn input() -> RoutingSnapshot {
    let report=serde_json::from_value(serde_json::json!({
  "hostname":"fixture", "agent_version":"fixture",
  "hardware":{"processor":"x64","threads":8,"memory_bytes":32_000_000_000u64,"kvm":true,"disks":[{"name":"/","capacity_bytes":1_000_000_000_000u64,"free_bytes":800_000_000_000u64}]},
  "capacity":3,"job_disk_bytes":150_000_000_000u64,"slots":[],"runners":[],"drain":"active",
  "cache":{"used_bytes":0,"capacity_bytes":0,"restores_last_7_days":0,"hits_last_7_days":0,"jobs":[]},"docker_disks":{"leased_bytes":0,"repositories":0,"used_bytes":0},"images":{"available":[{"image_id":"fixture","root_disk_bytes":60_263_759_872u64}],"newest":"fixture"}
 })).unwrap();
    RoutingSnapshot {
        workspace_id: Uuid::from_u128(1),
        organizations: vec![organization(2)],
        labels: vec![Label {
            id: Uuid::from_u128(4),
            name: "ciproxy".to_owned(),
            size: VmSize {
                vcpus: 2,
                memory_gb: 8,
                disk_gb: 80,
            },
            allow_machines: true,
            allow_boat: false,
            when_nothing_fits: WhenNothingFits::Wait,
        }],
        machines: vec![MachineSnapshot {
            machine: Machine {
                id: Uuid::from_u128(3),
                hostname: "fixture".to_owned(),
                processor: "x64".to_owned(),
                hardware_threads: 8,
                memory_bytes: 32_000_000_000,
                kvm_available: true,
                slots: 3,
                warm_runners: 2,
                draining: false,
                last_contact_at: Some(at()),
                agent_version: "fixture".to_owned(),
                disks: vec![],
                removed_at: None,
            },
            report: Some(ReceivedReport {
                report,
                received_at: at(),
            }),
        }],
        runners: vec![],
        jobs: vec![],
        settings: Settings {
            overflow_enabled: false,
            overflow_wait: OverflowWait::Seconds30,
            boat_spend_limit_usd_micros: None,
        },
    }
}

pub(super) fn organization(id: u128) -> Organization {
    Organization {
        id: Uuid::from_u128(id),
        workspace_id: Uuid::from_u128(1),
        github_id: id as i64,
        login: format!("org-{id}"),
        installation_id: id as i64,
        app_state: AppState::Active,
        repositories: RepositoryAccess::AllPrivate,
        removed_at: None,
        delete_after: None,
    }
}

pub(super) fn job(id: u128, org: u128, age_hours: i64) -> GithubJob {
    GithubJob {
        id: Uuid::from_u128(id),
        organization_id: Uuid::from_u128(org),
        scale_set_id: Uuid::from_u128(5),
        label_id: Uuid::from_u128(4),
        actions_job_id: None,
        request_id: 0,
        run_id: 7,
        repository: "org/repo".to_owned(),
        name: "build".to_owned(),
        workflow_ref: "org/repo/.github/workflows/ci.yml@refs/heads/main".to_owned(),
        phase: DemandPhase::Completed,
        result: Some("success".to_owned()),
        assigned_at: Some(at() - chrono::Duration::hours(age_hours)),
        started_at: None,
        completed_at: Some(at()),
        runner_github_id: None,
        facts: RunFacts {
            id: 7,
            repository: RepositoryIdentity {
                id: 8,
                full_name: "org/repo".to_owned(),
            },
            event: "push".to_owned(),
            head_branch: Some("main".to_owned()),
            head_sha: "fixture".to_owned(),
            head_repository: None,
        },
    }
}
