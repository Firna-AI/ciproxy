//! Job message durability, historical resource snapshots, and retention.

use chrono::Duration;
use ciproxy_store_interface::jobs::{CacheSource, JobResult, JobState, MachineHandledCacheCounts};
use ciproxy_store_interface::runners::RunnerPlace;
use ciproxy_store_interface::vm_size::VmSize;
use uuid::Uuid;

use super::support::{Database, at, job, machine, runner};

#[test]
fn job_messages_deduplicate_and_finished_metadata_expires_after_ninety_days() {
    let database = Database::new();
    let store = &database.store;
    let owner = database.workspace();
    let organization = database.organization(&owner);
    let (mut label, scale_set) = database.label_and_scale_set(&organization);
    let machine = machine();
    store.create_machine(owner.id, &machine).unwrap();
    let mut runner = runner(
        &label,
        &scale_set,
        RunnerPlace::Machine {
            machine_id: machine.id,
            first_slot: 0,
        },
    );
    runner.size.disk_bytes = 82_644_443_136;
    store.create_runner(owner.id, &runner).unwrap();
    assert_eq!(
        store.runners(owner.id, organization.id).unwrap()[0]
            .size
            .disk_bytes,
        82_644_443_136
    );
    let mut completed = job(&label, &scale_set);
    assert!(store.insert_job(owner.id, &completed).unwrap());
    completed.runner_id = Some(runner.id);
    completed.state = JobState::Completed;
    completed.result = Some(JobResult::Success);
    completed.started_at = Some(at() + Duration::seconds(15));
    completed.completed_at = Some(at() + Duration::minutes(2));
    completed.routing_reason = Some("A warm runner was ready.".to_owned());
    completed.cache_source = Some(CacheSource::MachineAndGithub);
    completed.machine_handled_cache_counts = Some(MachineHandledCacheCounts {
        restored_entries: 3,
        restored_bytes: 2_100_000_000,
    });
    store.update_job(owner.id, &completed).unwrap();
    let foreign_workspace = database.workspace();
    let foreign_organization = database.organization(&foreign_workspace);
    let (foreign_label, foreign_scale) = database.label_and_scale_set(&foreign_organization);
    let mut foreign_job = job(&foreign_label, &foreign_scale);
    foreign_job.state = JobState::Completed;
    foreign_job.completed_at = completed.completed_at;
    store
        .insert_job(foreign_workspace.id, &foreign_job)
        .unwrap();
    let mut duplicate = completed.clone();
    duplicate.id = Uuid::new_v4();
    duplicate.state = JobState::Queued;
    duplicate.name = "duplicate must not replace metadata".to_owned();
    assert!(!store.insert_job(owner.id, &duplicate).unwrap());
    assert_eq!(
        store.jobs(owner.id, organization.id).unwrap(),
        vec![completed.clone()]
    );
    label.size = VmSize {
        vcpus: 4,
        memory_gb: 16,
        disk_gb: 80,
    };
    store.update_label(owner.id, &label).unwrap();
    assert_eq!(
        store.runners(owner.id, organization.id).unwrap(),
        vec![runner]
    );
    let cache_cases = [
        (
            "github measured",
            Some(CacheSource::Github),
            Some(MachineHandledCacheCounts {
                restored_entries: 3,
                restored_bytes: 2_100_000_000,
            }),
        ),
        (
            "zero measured",
            None,
            Some(MachineHandledCacheCounts {
                restored_entries: 0,
                restored_bytes: 0,
            }),
        ),
        ("github direct", Some(CacheSource::Github), None),
    ];
    for (scenario, source, counts) in cache_cases {
        let mut observed = completed.clone();
        observed.cache_source = source;
        observed.machine_handled_cache_counts = counts;
        let result = store.update_job(owner.id, &observed);
        assert!(matches!(result, Ok(())), "{scenario}: {result:?}");
        assert_eq!(
            store.jobs(owner.id, organization.id).unwrap(),
            vec![observed],
            "{scenario}"
        );
    }
    let mut waiting = job(&label, &scale_set);
    waiting.github_id = 43;
    store.insert_job(owner.id, &waiting).unwrap();
    let expiry = completed.completed_at.unwrap() + Duration::days(90);
    assert_eq!(
        store
            .purge_finished_jobs(owner.id, expiry - Duration::seconds(1))
            .unwrap(),
        0
    );
    assert_eq!(store.purge_finished_jobs(owner.id, expiry).unwrap(), 1);
    assert_eq!(
        store.jobs(owner.id, organization.id).unwrap(),
        vec![waiting]
    );
    assert_eq!(
        store
            .jobs(foreign_workspace.id, foreign_organization.id)
            .unwrap(),
        vec![foreign_job]
    );
}
