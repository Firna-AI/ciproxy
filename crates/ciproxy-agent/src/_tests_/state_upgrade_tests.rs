//! Saved VM facts recover only the obsolete null-identity shape.

use std::collections::BTreeSet;

use ciproxy_protocol::guest::JobPreparation;
use serde_json::{Value, json};

use crate::cache_authority::CacheAuthority;
use crate::fixtures::{identity, trusted_facts, vm};
use crate::state::{MachineState, Mode, PreparationState};

#[test]
fn legacy_null_job_facts_are_discarded_without_changing_running_vm_state() {
    let mut record = vm("running", true);
    record.preparation = PreparationState::Completed(JobPreparation {
        cache_ready: false,
        docker_attached: false,
    });
    record.cache_authority = CacheAuthority::GitHubOnly;
    record.job_facts = Some(trusted_facts());
    let state = MachineState {
        identity: Some(identity()),
        mode: Mode::Draining,
        pending_networks: BTreeSet::from([3, 7]),
        failed_starts: BTreeSet::from(["failed-start".into()]),
        runners: vec![record],
        ..MachineState::default()
    };
    let current = serde_json::to_value(&state).unwrap();
    assert_eq!(
        serde_json::from_value::<MachineState>(current.clone()).map_err(|error| error.classify()),
        Ok(state.clone())
    );

    let mut legacy = current.clone();
    let assignment = legacy["runners"][0]["job_facts"]["assignment"]
        .as_object_mut()
        .unwrap();
    assignment.remove("job_ids");
    assignment.insert("job_id".into(), Value::Null);
    let mut expected = state;
    expected.runners[0].job_facts = None;
    assert_eq!(
        serde_json::from_value::<MachineState>(legacy).map_err(|error| error.classify()),
        Ok(expected.clone())
    );
    assert_eq!(
        serde_json::from_value::<MachineState>(serde_json::to_value(&expected).unwrap())
            .map_err(|error| error.classify()),
        Ok(expected)
    );

    for (case, job_ids, legacy_null) in [
        (
            "partial current pair",
            json!({"worker_job_id": "job-1"}),
            false,
        ),
        (
            "partial pair with legacy null",
            json!({"worker_job_id": "job-1"}),
            true,
        ),
        ("null pair with legacy null", Value::Null, true),
    ] {
        let mut invalid = current.clone();
        let assignment = invalid["runners"][0]["job_facts"]["assignment"]
            .as_object_mut()
            .unwrap();
        assignment.insert("job_ids".into(), job_ids);
        if legacy_null {
            assignment.insert("job_id".into(), Value::Null);
        }
        assert_eq!(
            serde_json::from_value::<MachineState>(invalid)
                .map(|_| ())
                .map_err(|error| error.classify()),
            Err(serde_json::error::Category::Data),
            "{case}"
        );
    }
}
