//! App ownership scenarios use the real store and SDK with fixed identities.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use chrono::{DateTime, Utc};
use ciproxy_github::app::webhook::{Actor, Installation, InstallationAction, InstallationEvent};
use ciproxy_github::app::{AppId, AppService};
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::registrations::{
    RegistrationRequest, RegistrationService, RunnerRegistrations,
};
use ciproxy_github::runtime::Runtime;
use ciproxy_protocol::resources::VmSize;
use ciproxy_store_interface::billing::state::Tier;
use ciproxy_store_interface::dashboard::records::Membership;
use ciproxy_store_interface::github::{DemandPhase, GithubJob, RepositoryIdentity, RunFacts};
use ciproxy_store_interface::organizations::{AppState, Organization, RepositoryAccess};
use ciproxy_store_interface::runners::{Runner, RunnerPlace, RunnerState};
use ciproxy_store_interface::scale_sets::{RegistrationTarget, ScaleSet};
use ciproxy_store_interface::store::Store;
use ciproxy_store_interface::workspaces::Workspace;
use uuid::Uuid;

use super::super::app_fixture::{Fixture, at, installed};
use super::super::resource_fixture::seed_target_authority;
use super::super::support::{Factory, id};

struct Inputs(AtomicU64);

impl Runtime for Inputs {
    fn now(&self) -> DateTime<Utc> {
        at()
    }

    fn uuid(&self) -> Uuid {
        id(u128::from(self.0.fetch_add(1, Ordering::SeqCst)))
    }
}

pub(super) fn app(store: Arc<dyn Store>, fixture: &Fixture, origin: &str) -> AppService {
    let _configured = fixture.app(store.clone());
    let adapter = Arc::new(StoreAdapter::new(
        store,
        Arc::new(Inputs(AtomicU64::new(1000))),
    ));
    AppService {
        app_id: AppId::new("3571".to_owned()),
        leases: adapter.clone(),
        installations: fixture.state.clone(),
        store: adapter.clone(),
        outbox: adapter,
        rest: fixture.remote.clone(),
        scale_sets: Arc::new(Factory(origin.to_owned())),
        authority: fixture.authority.clone(),
        lifecycle: fixture.output.clone(),
        runtime: Arc::new(Inputs(AtomicU64::new(2000))),
    }
}

pub(super) fn event(action: InstallationAction, installation_id: i64) -> InstallationEvent {
    InstallationEvent {
        action,
        installation: Installation {
            id: installation_id,
            account: installed(installation_id, false).account,
        },
        requester: Some(Actor { id: 41 }),
        sender: Actor { id: 99 },
    }
}

pub(super) fn target_workspace(store: &dyn Store) -> Uuid {
    let workspace = Workspace {
        id: id(90),
        name: "target".to_owned(),
        delete_after: None,
    };
    store.create_workspace(&workspace).unwrap();
    seed_target_authority(store, workspace.id, 41);
    let mut label = store.labels(id(1)).unwrap().remove(0);
    for (label_id, name) in [(91, "ciproxy"), (92, "ciproxy-extra")] {
        label.id = id(label_id);
        label.name = name.to_owned();
        store.create_label(workspace.id, &label).unwrap();
    }
    store
        .install_organization(&Organization {
            id: id(95),
            workspace_id: id(1),
            github_id: 6000,
            login: "original-member".to_owned(),
            installation_id: 6001,
            app_state: AppState::Active,
            repositories: RepositoryAccess::AllPrivate,
            removed_at: None,
            delete_after: None,
        })
        .unwrap();
    store
        .ensure_dashboard_initial_admin(id(1), 41, at())
        .unwrap();
    store
        .save_dashboard_user(
            &store.dashboard_user_by_github(41).unwrap().unwrap(),
            &[
                Membership {
                    github_id: 5000,
                    owner: false,
                },
                Membership {
                    github_id: 6000,
                    owner: false,
                },
            ],
        )
        .unwrap();
    let mut billing = store.try_lock_billing(id(1)).unwrap().unwrap();
    let mut record = billing.record().unwrap().state;
    record.tier = Tier::Team;
    billing
        .commit(Some(&record), None, false, false, at())
        .unwrap();
    workspace.id
}

pub(super) async fn register_runner(
    store: Arc<dyn Store>,
    origin: &str,
    workspace: Uuid,
    set: &ScaleSet,
    runner_id: u128,
    state: RunnerState,
) -> Runner {
    let service = RegistrationService {
        store: Arc::new(StoreAdapter::new(
            store.clone(),
            Arc::new(Inputs(AtomicU64::new(3000))),
        )),
        scale_sets: Arc::new(Factory(origin.to_owned())),
    };
    let mut runner = Runner {
        id: id(runner_id),
        organization_id: set.organization_id,
        label_id: set.label_id,
        scale_set_id: set.id,
        github_id: None,
        state: RunnerState::Starting,
        size: VmSize {
            vcpu: 2,
            memory_bytes: 8_000_000_000,
            disk_bytes: 80_000_000_000,
        },
        place: RunnerPlace::Boat {
            key_id: None,
            sandbox_id: format!("runner-{runner_id}"),
            size_name: "small".to_owned(),
        },
    };
    store.create_runner(workspace, &runner).unwrap();
    let registered = service
        .create(
            RegistrationRequest {
                workspace_id: workspace,
                target: RegistrationTarget::from(set),
            },
            &format!("runner-{runner_id}"),
        )
        .await
        .unwrap();
    store
        .save_runner_registration(
            workspace,
            runner.id,
            registered.target,
            registered.runner_id,
        )
        .unwrap();
    store
        .update_runner(
            workspace,
            set.organization_id,
            runner.id,
            state,
            Some(registered.runner_id),
        )
        .unwrap();
    runner.github_id = Some(registered.runner_id);
    runner.state = state;
    runner
}

pub(super) fn running_job(set: &ScaleSet, runner: &Runner) -> GithubJob {
    GithubJob {
        id: id(400),
        organization_id: set.organization_id,
        scale_set_id: set.id,
        label_id: set.label_id,
        actions_job_id: None,
        request_id: 0,
        run_id: 31,
        repository: "customer/private".to_owned(),
        name: "running-before-transfer".to_owned(),
        workflow_ref: "customer/private/.github/workflows/build.yml@refs/heads/main".to_owned(),
        phase: DemandPhase::Started,
        result: None,
        assigned_at: Some(at()),
        started_at: Some(at()),
        completed_at: None,
        runner_github_id: runner.github_id,
        facts: RunFacts {
            id: 31,
            repository: RepositoryIdentity {
                id: 23,
                full_name: "customer/private".to_owned(),
            },
            event: "push".to_owned(),
            head_branch: Some("main".to_owned()),
            head_sha: "0123456789abcdef".to_owned(),
            head_repository: None,
        },
    }
}
