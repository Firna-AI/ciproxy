//! Public store fixtures for retained runner targets and scoped started jobs.

use ciproxy_protocol::resources::VmSize;
use ciproxy_store_interface::github::{DemandPhase, GithubJob, RepositoryIdentity, RunFacts};
use ciproxy_store_interface::organizations::{AppState, Organization, RepositoryAccess};
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use ciproxy_store_interface::runners::{Runner, RunnerPlace, RunnerState};
use ciproxy_store_interface::scale_sets::{RegistrationTarget, ScaleSet};
use ciproxy_store_interface::store::Store;
use ciproxy_store_interface::workspaces::Workspace;
use uuid::Uuid;

use super::support::id;

/// Create an active registration and its label in a new workspace.
pub(super) fn registration(store: &dyn Store, base: u128, github_id: i64) -> ScaleSet {
    store
        .create_workspace(&Workspace {
            id: id(base),
            name: format!("workspace-{base}"),
            delete_after: None,
        })
        .unwrap();
    store
        .install_organization(&Organization {
            id: id(base + 3),
            workspace_id: id(base),
            github_id,
            login: if github_id == 17 {
                "customer".to_owned()
            } else {
                "other-organization".to_owned()
            },
            installation_id: i64::try_from(base + 9).unwrap(),
            app_state: AppState::Active,
            repositories: RepositoryAccess::AllPrivate,
            removed_at: None,
            delete_after: None,
        })
        .unwrap();
    let mut label = store.labels(id(1)).unwrap().remove(0);
    label.id = id(base + 1);
    store.create_label(id(base), &label).unwrap();
    let binding = ScaleSet {
        id: id(base + 4),
        organization_id: id(base + 3),
        label_id: label.id,
        github_id: 53,
        runner_group_id: 47,
    };
    store.create_scale_set(id(base), &binding).unwrap();
    binding
}

/// Save one runner with its exact captured registration target.
pub(super) fn runner(
    store: &dyn Store,
    workspace: Uuid,
    binding: &ScaleSet,
    github_id: i64,
) -> Runner {
    let mut runner = Runner {
        id: id(400 + u128::try_from(github_id).unwrap()),
        organization_id: binding.organization_id,
        label_id: binding.label_id,
        scale_set_id: binding.id,
        github_id: None,
        state: RunnerState::Busy,
        size: VmSize {
            vcpu: 2,
            memory_bytes: 8_000_000_000,
            disk_bytes: 80_000_000_000,
        },
        place: RunnerPlace::Overflow {
            provider: OverflowProvider::Boat,
            key_id: None,
            sandbox_id: format!("runner-{github_id}"),
            size_name: "small".to_owned(),
        },
    };
    store.create_runner(workspace, &runner).unwrap();
    store
        .save_runner_registration(
            workspace,
            runner.id,
            RegistrationTarget::from(binding),
            github_id,
        )
        .unwrap();
    runner.github_id = Some(github_id);
    runner
}

/// Save a started attempt with a known start and recorded GitHub runner ID.
pub(super) fn started(
    store: &dyn Store,
    workspace: Uuid,
    binding: &ScaleSet,
    runner_id: i64,
) -> GithubJob {
    let job = GithubJob {
        id: id(500 + u128::try_from(runner_id).unwrap()),
        organization_id: binding.organization_id,
        scale_set_id: binding.id,
        label_id: binding.label_id,
        actions_job_id: None,
        request_id: 0,
        run_id: 31,
        repository: "customer/private".to_owned(),
        name: "started-before-binding-replacement".to_owned(),
        workflow_ref: "customer/private/.github/workflows/build.yml@refs/heads/main".to_owned(),
        phase: DemandPhase::Started,
        result: None,
        assigned_at: Some("2026-11-04T11:59:59.000000001Z".parse().unwrap()),
        started_at: Some("2026-11-04T12:00:00.000000001Z".parse().unwrap()),
        completed_at: None,
        runner_github_id: Some(runner_id),
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
    };
    store
        .save_github_job(workspace, &job, "2026-11-04T12:00:00Z".parse().unwrap())
        .unwrap();
    job
}
