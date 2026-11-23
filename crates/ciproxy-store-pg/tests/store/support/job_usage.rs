//! Job sample, registered runner, and deletion fixtures.

use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::usage::{JobStep, JobStepTiming, JobUsage};
use ciproxy_schema::job_usage::job_usage_steps;
use ciproxy_store_interface::job_usage::StoredJobUsage;
use ciproxy_store_interface::labels::Label;
use ciproxy_store_interface::runners::{Runner, RunnerPlace};
use ciproxy_store_interface::scale_sets::ScaleSet;
use diesel::prelude::*;
use uuid::Uuid;

use super::{Database, at, runner};

/// Check that the sample and its step rows are gone.
pub(in crate::store) fn assert_job_usage_deleted(
    database: &mut Database,
    workspace_id: Uuid,
    runner_id: Uuid,
) {
    assert_eq!(
        database.store.job_usage(workspace_id, runner_id).unwrap(),
        None
    );
    assert_eq!(
        job_usage_steps::table
            .filter(job_usage_steps::workspace_id.eq(workspace_id))
            .filter(job_usage_steps::runner_id.eq(runner_id))
            .count()
            .get_result::<i64>(&mut database.connection)
            .unwrap(),
        0
    );
}

/// Register a Boat runner with its own GitHub identity.
pub(in crate::store) fn boat_runner(
    database: &Database,
    workspace_id: Uuid,
    label: &Label,
    scale_set: &ScaleSet,
) -> Runner {
    let mut runner = runner(
        label,
        scale_set,
        RunnerPlace::Boat {
            key_id: None,
            sandbox_id: Uuid::new_v4().to_string(),
            size_name: "small".to_owned(),
        },
    );
    runner.github_id =
        Some(i64::from_be_bytes(runner.id.as_bytes()[..8].try_into().unwrap()) & i64::MAX);
    database.store.create_runner(workspace_id, &runner).unwrap();
    runner
}

/// A sample report with two different points and one timed step.
pub(in crate::store) fn sample_report() -> StoredJobUsage {
    StoredJobUsage {
        received_at: at(),
        is_final: false,
        usage: JobUsage {
            platform: Platform::LinuxX64,
            first_point_at_ms: 1_793_793_600_000,
            point_seconds: 2,
            cpu_milli_vcpu: vec![25, 1_275],
            peak_memory_mib: vec![1_024, 3_072],
            oom_kills: Some(vec![0, 2]),
            steps: vec![JobStep {
                name: "Run actions/checkout@v4".to_owned(),
                timing: JobStepTiming::Ran {
                    started_at_ms: Some(1_793_793_600_123),
                    ended_at_ms: Some(1_793_793_602_456),
                },
            }],
        },
    }
}
