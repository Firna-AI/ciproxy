//! Literal GitHub metadata with nanosecond assignment precision.

use ciproxy_store_interface::github::{DemandPhase, GithubJob, RepositoryIdentity, RunFacts};
use ciproxy_store_interface::labels::Label;
use ciproxy_store_interface::scale_sets::ScaleSet;
use uuid::Uuid;

pub(super) fn demand(label: &Label, scale_set: &ScaleSet) -> GithubJob {
    GithubJob {
        id: Uuid::from_u128(71),
        organization_id: scale_set.organization_id,
        scale_set_id: scale_set.id,
        label_id: label.id,
        actions_job_id: None,
        request_id: 0,
        run_id: 31,
        repository: "customer/private".to_owned(),
        name: "test".to_owned(),
        workflow_ref: "shared/.github/workflows/reuse.yml@refs/heads/main".to_owned(),
        phase: DemandPhase::Assigned,
        result: None,
        assigned_at: Some("2026-11-04T10:46:21.620395766Z".parse().unwrap()),
        started_at: None,
        completed_at: None,
        runner_github_id: None,
        facts: RunFacts {
            id: 31,
            repository: RepositoryIdentity {
                id: 23,
                full_name: "customer/private".to_owned(),
            },
            event: "pull_request_target".to_owned(),
            head_branch: Some("feature".to_owned()),
            head_sha: "0123456789abcdef".to_owned(),
            head_repository: None,
        },
    }
}
