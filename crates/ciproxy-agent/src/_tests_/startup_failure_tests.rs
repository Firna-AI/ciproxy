//! Root faults release only the failed startup's durable slots and job disk.

use std::sync::{Arc, Mutex};

use ciproxy_protocol::identifiers::ImageId;
use ciproxy_protocol::reports::SlotOccupancy;
use ciproxy_protocol::resources::VmSize;
use ciproxy_protocol::runners::{RunnerReport, RunnerState};
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{CacheMock, StoreMock, VmsMock};
use crate::controller::Agent;
use crate::fixtures::{
    controller, hardware, identity, report_dependencies_for, spec, start_image, vm,
};
use crate::root_failure_fixtures::{RootFailure, Stage, error_facts, expected_trace, layout};
use crate::state::{MachineState, VmRecord};

#[derive(Debug, Eq, PartialEq)]
enum Event {
    Saved(MachineState),
    Budget(u64),
    Started(VmRecord),
    Cleanup(VmRecord),
    Restored(VmRecord),
}

#[tokio::test]
async fn root_preparation_failures_clean_up_the_new_vm_and_release_its_reservation() {
    let failure = RootFailure::new();
    for stage in Stage::ALL {
        failure.reset(stage);
        let events = Arc::new(Mutex::new(Vec::new()));
        let initial = MachineState {
            identity: Some(identity()),
            runners: vec![vm("warm-existing", false)],
            ..MachineState::default()
        };
        let mut runner = spec();
        runner.slot_indices = vec![1, 2];
        runner.size.vcpu = 4;
        runner.size.memory_bytes = 16_000_000_000;
        runner.size.disk_bytes = 100_000_000_000;
        let mut allocated = vm("new", false);
        allocated.network_index = 2;
        allocated.report = RunnerReport {
            configuration_lost: false,
            slot_indices: vec![1, 2],
            runner_id: runner.runner_id.clone(),
            organization_id: runner.organization_id.clone(),
            label: runner.label.clone(),
            requested_size: runner.size,
            size: VmSize {
                disk_bytes: 163_373_842_432,
                ..runner.size
            },
            image_id: ImageId("image-1".to_owned()),
            state: RunnerState::Starting,
        };
        let mut reserved = initial.clone();
        reserved.runners.push(allocated.clone());
        let mut machine = hardware();
        machine.disks[0].capacity_bytes = 512_000_000_000;
        machine.disks[0].free_bytes = 400_000_000_000;
        let saving = events.clone();
        let budgets = events.clone();
        let starts = events.clone();
        let cleanups = events.clone();
        let restores = events.clone();
        let root = failure.clone();
        let refreshing = events.clone();
        let mock = Unimock::new((
            VmsMock::networks
                .some_call(matching!())
                .returns(Ok(Vec::new())),
            VmsMock::remove_network
                .some_call(matching!(2))
                .returns(Ok(())),
            report_dependencies_for(machine),
            start_image(),
            VmsMock::allocate
                .each_call(matching!(_, _))
                .answers(&|_, report, _| {
                    let mut allocated = vm("new", false);
                    allocated.network_index = 2;
                    allocated.report = report;
                    Ok(allocated)
                })
                .at_least_times(0),
            StoreMock::save
                .each_call(matching!(_))
                .answers_arc(Arc::new(move |_, state| {
                    saving.lock().unwrap().push(Event::Saved(state.clone()));
                    Ok(())
                }))
                .at_least_times(0),
            CacheMock::refresh_budget
                .each_call(matching!(_))
                .answers_arc(Arc::new(move |_, bytes| {
                    refreshing.lock().unwrap().push(Event::Budget(bytes));
                    Ok(())
                }))
                .at_least_times(0),
            CacheMock::budget
                .each_call(matching!(_))
                .answers_arc(Arc::new(move |_, bytes| {
                    budgets.lock().unwrap().push(Event::Budget(bytes));
                    Ok(())
                }))
                .at_least_times(0),
            VmsMock::start
                .each_call(matching!(_, _))
                .answers_arc(Arc::new(move |_, record, _| {
                    starts.lock().unwrap().push(Event::Started(record.clone()));
                    let paths = layout();
                    root.prepare_ready(
                        &paths.image(&record.report.image_id).join("rootfs.ext4"),
                        &paths.jail_root(record).join("rootfs.ext4"),
                        record.report.size.disk_bytes,
                    )
                }))
                .at_least_times(0),
            VmsMock::stop
                .each_call(matching!(_))
                .answers_arc(Arc::new(move |_, record| {
                    cleanups
                        .lock()
                        .unwrap()
                        .push(Event::Cleanup(record.clone()));
                    Ok(())
                }))
                .at_least_times(0),
            VmsMock::restore
                .each_call(matching!(_))
                .answers_arc(Arc::new(move |_, record| {
                    restores
                        .lock()
                        .unwrap()
                        .push(Event::Restored(record.clone()));
                    Ok(())
                }))
                .at_least_times(0),
        ));
        let agent = controller(initial.clone(), mock);
        assert_eq!(
            agent.start(runner).await.map_err(error_facts),
            Err(stage.expected_error()),
            "{stage:?}",
        );
        let paths = layout();
        let image = paths.image(&allocated.report.image_id).join("rootfs.ext4");
        let copy = paths.jail_root(&allocated).join("rootfs.ext4");
        assert_eq!(
            failure.trace(),
            expected_trace(stage, &image, &copy, 163_373_842_432),
            "{stage:?}"
        );
        let mut cleanup_pending = initial.clone();
        cleanup_pending.pending_networks.insert(2);
        assert_eq!(*agent.state.lock().await, cleanup_pending, "{stage:?}");
        agent.reconcile().await.unwrap();
        assert_eq!(*agent.state.lock().await, initial, "{stage:?}");
        let report = agent.report().await.unwrap();
        assert_eq!(
            report.runners,
            vec![initial.runners[0].report.clone()],
            "{stage:?}"
        );
        assert_eq!(
            report
                .slots
                .iter()
                .filter(|slot| slot.occupancy == SlotOccupancy::Free)
                .count(),
            5,
            "{stage:?}"
        );
        assert_eq!(
            *events.lock().unwrap(),
            vec![
                Event::Saved(reserved),
                Event::Budget(34_726_544_896),
                Event::Started(allocated.clone()),
                Event::Cleanup(allocated),
                Event::Saved(cleanup_pending),
                Event::Restored(initial.runners[0].clone()),
                Event::Budget(198_100_387_328),
                Event::Saved(initial.clone()),
            ],
            "{stage:?}"
        );
    }
}

#[tokio::test]
async fn failed_cache_reclaim_releases_the_reservation_before_retrying_the_vm() {
    let mock = Unimock::new((
        report_dependencies_for(hardware()),
        start_image(),
        VmsMock::allocate
            .each_call(matching!(_, _))
            .answers(&|_, report, _| {
                let mut allocated = vm("runner-1", false);
                allocated.report = report;
                Ok(allocated)
            }),
        StoreMock::save
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        CacheMock::budget.stub(|each| {
            each.call(matching!(_))
                .answers(&|_, _| Err(crate::error::Error::CacheUnavailable))
                .once()
                .then()
                .answers(&|_, _| Ok(()));
        }),
        VmsMock::stop.some_call(matching!(_)).returns(Ok(())),
        CacheMock::release.some_call(matching!(_)).returns(Ok(())),
        VmsMock::start.some_call(matching!(_, _)).returns(Ok(())),
    ));
    let agent = controller(MachineState::default(), mock);
    assert_eq!(
        agent
            .start(spec())
            .await
            .map_err(|error| matches!(error, crate::error::Error::CacheUnavailable)),
        Err(true)
    );
    assert_eq!(agent.state.lock().await.runners, Vec::<VmRecord>::new());
    agent.start(spec()).await.unwrap();
    assert_eq!(agent.state.lock().await.runners.len(), 1);
}
