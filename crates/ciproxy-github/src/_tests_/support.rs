//! Literal service fixtures and mock composition without external IO.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use ciproxy_store_interface::github::{Repository, RepositoryIdentity, RunFacts};
use ciproxy_store_interface::labels::{Label, WhenNothingFits};
use ciproxy_store_interface::organizations::{AppState, Organization, RepositoryAccess};
use ciproxy_store_interface::scale_sets::{ScaleSet, ScaleSetState};
use ciproxy_store_interface::vm_size::VmSize;
use unimock::Unimock;
use uuid::Uuid;

use crate::app::AppService;
use crate::app::webhook::{
    Account, AccountKind, Actor, Installation, InstallationAction, InstallationEvent,
};
use crate::persistence::session::{ListenerContext, ListenerTarget};

pub(super) fn at() -> DateTime<Utc> {
    "2026-11-04T12:00:00Z".parse().unwrap()
}
pub(super) fn id(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

pub(super) fn organization() -> Organization {
    Organization {
        id: id(5),
        workspace_id: id(1),
        github_id: 17,
        login: "customer".to_owned(),
        installation_id: 19,
        app_state: AppState::Active,
        repositories: RepositoryAccess::AllPrivate,
        removed_at: None,
        delete_after: None,
    }
}

pub(super) fn labels() -> Vec<Label> {
    [
        (2, "ciproxy", 2, 8, 20),
        (3, "ciproxy-4vcpu", 4, 16, 20),
        (4, "ciproxy-8vcpu", 8, 32, 60),
    ]
    .into_iter()
    .map(|(identity, name, vcpus, memory_gb, disk_gb)| Label {
        id: id(identity),
        name: name.to_owned(),
        size: VmSize {
            vcpus,
            memory_gb,
            disk_gb,
        },
        allow_machines: true,
        allow_boat: true,
        when_nothing_fits: WhenNothingFits::Wait,
    })
    .collect()
}

pub(super) fn event(action: InstallationAction) -> InstallationEvent {
    InstallationEvent {
        action,
        installation: Installation {
            id: 19,
            account: Account {
                id: 17,
                login: "customer".to_owned(),
                kind: AccountKind::Organization,
            },
        },
        requester: Some(Actor { id: 41 }),
        sender: Actor { id: 99 },
    }
}

pub(super) fn repositories() -> Vec<Repository> {
    vec![
        Repository {
            id: 23,
            full_name: "customer/private".to_owned(),
            private: true,
            default_branch: "trunk".to_owned(),
        },
        Repository {
            id: 29,
            full_name: "customer/public".to_owned(),
            private: false,
            default_branch: "main".to_owned(),
        },
    ]
}

pub(super) fn facts() -> RunFacts {
    RunFacts {
        id: 31,
        repository: RepositoryIdentity {
            id: 23,
            full_name: "customer/private".to_owned(),
        },
        event: "pull_request_target".to_owned(),
        head_branch: Some("untrusted-feature".to_owned()),
        head_sha: "0123456789abcdef".to_owned(),
        head_repository: None,
    }
}

pub(super) fn app(mock: Unimock) -> AppService {
    let mock = Arc::new(mock);
    AppService {
        leases: mock.clone(),
        installations: mock.clone(),
        store: mock.clone(),
        outbox: mock.clone(),
        rest: mock.clone(),
        scale_sets: mock.clone(),
        authority: mock.clone(),
        lifecycle: mock.clone(),
        runtime: mock,
    }
}

pub(super) fn target() -> ListenerTarget {
    ListenerTarget {
        workspace_id: id(1),
        organization_id: id(5),
        scale_set_id: id(6),
    }
}

pub(super) fn context() -> ListenerContext {
    ListenerContext {
        state: ScaleSetState::Active,
        has_tracked_jobs: true,
        organization: organization(),
        scale_set: ScaleSet {
            id: id(6),
            organization_id: id(5),
            label_id: id(2),
            github_id: 53,
            runner_group_id: 47,
        },
    }
}
