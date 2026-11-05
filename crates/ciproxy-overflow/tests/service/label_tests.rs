//! Label policy is current across provider waits and durable startup entries.

use ciproxy_overflow::api::{Overflow, Retirement, RunnerAuthorityMock, StartOutcome, WaitReason};
use ciproxy_overflow::maintenance::LaunchState;
use ciproxy_store_interface::labels::WhenNothingFits;
use ciproxy_store_interface::overflow::record::StartPhase;
use sandbox_interface::SandboxState;
use unimock::{MockFn, matching};

use super::authority_actions::{Entry, reconcile};
use super::authority_support::{DisableAt, Provider, disable};
use super::fixtures::{id, record, request};
use super::harness::{Harness, Runtime};
use super::label_hooks::{Change, LabelStage, change_label};

#[tokio::test]
async fn current_label_policy_stops_startup_after_each_provider_wait() {
    for change in [Change::BoatOff, Change::Cpu, Change::Memory, Change::Disk] {
        for stage in [
            LabelStage::Account,
            LabelStage::Capacity,
            LabelStage::Usage,
            LabelStage::Create,
            LabelStage::Inspect,
            LabelStage::Bootstrap,
            LabelStage::Prepare,
            LabelStage::Input,
        ] {
            let provider = Provider::new(DisableAt::Never);
            let harness = Harness::new(provider.backend, (), Runtime::Start);
            harness.set_checked("2026-11-05T11:00:00Z".parse().unwrap());
            if stage == LabelStage::Usage {
                let mut running = record(StartPhase::Running);
                running.delivery_id = id(21);
                running.runner_id = id(22);
                harness.seed(running);
            }
            provider
                .label
                .set(stage, change, harness.database.store.clone());
            let outcome = harness.service.start(id(1), request()).await.unwrap();
            assert_eq!(
                *provider.label.applied.lock().unwrap(),
                1,
                "{change:?}, {stage:?}"
            );
            let counts = provider.counts.lock().unwrap();
            let before_reservation = matches!(
                stage,
                LabelStage::Account | LabelStage::Capacity | LabelStage::Usage
            );
            assert_eq!(
                counts.creates,
                usize::from(!before_reservation),
                "{change:?}, {stage:?}"
            );
            assert_eq!(
                counts.inputs,
                usize::from(stage == LabelStage::Input),
                "{change:?}, {stage:?}"
            );
            assert_eq!(counts.launches, 0, "{change:?}, {stage:?}");
            assert_eq!(
                counts.deletes,
                usize::from(!before_reservation),
                "{change:?}, {stage:?}"
            );
            assert_eq!(
                outcome == StartOutcome::Waiting(WaitReason::Disconnected),
                before_reservation
            );
            assert_eq!(
                matches!(outcome, StartOutcome::Ended(_)),
                !before_reservation
            );
            assert_eq!(
                harness.snapshot().starts.len(),
                usize::from(!before_reservation || stage == LabelStage::Usage)
            );
            let expected_phase = if stage == LabelStage::Usage {
                Some(StartPhase::Running)
            } else {
                (!before_reservation).then_some(StartPhase::Ended)
            };
            assert_eq!(
                harness.snapshot().starts.first().map(|start| start.phase),
                expected_phase
            );
        }
    }
}

#[tokio::test]
async fn changed_labels_withdraw_pending_resume_duplicate_and_inventory_authority() {
    for change in [Change::BoatOff, Change::Cpu, Change::Memory, Change::Disk] {
        for entry in [Entry::Resume, Entry::Duplicate, Entry::Cleanup] {
            for phase in [
                StartPhase::Creating,
                StartPhase::Unsent,
                StartPhase::Preparing,
                StartPhase::Writing,
                StartPhase::Launching,
            ] {
                let provider = Provider::new(DisableAt::Never);
                let kind = if phase == StartPhase::Unsent {
                    Runtime::Cancellation
                } else {
                    Runtime::Read
                };
                let harness = Harness::new(provider.backend, (), kind);
                let mut pending = record(phase);
                if matches!(phase, StartPhase::Unsent | StartPhase::Creating) {
                    pending.sandbox = None;
                    pending.start_usd = None;
                }
                harness.seed(pending);
                change_label(&harness.database.store, change);
                reconcile(&harness, entry).await;
                let counts = provider.counts.lock().unwrap();
                assert_eq!(counts.creates, 0, "{change:?}, {entry:?}, {phase:?}");
                assert_eq!(counts.inputs, 0, "{change:?}, {entry:?}, {phase:?}");
                assert_eq!(counts.launches, 0, "{change:?}, {entry:?}, {phase:?}");
                assert_eq!(
                    counts.deletes,
                    usize::from(phase != StartPhase::Unsent),
                    "{change:?}, {entry:?}, {phase:?}"
                );
                let expected = if phase == StartPhase::Unsent {
                    StartPhase::CancelledUnsent
                } else {
                    StartPhase::Ended
                };
                assert_eq!(harness.start(id(11)).phase, expected);
            }
        }
    }
}

#[tokio::test]
async fn label_changes_during_recovery_and_inventory_cancel_pending_startup() {
    for change in [Change::BoatOff, Change::Cpu, Change::Memory, Change::Disk] {
        for (stage, cleanup) in [
            (LabelStage::Recover, false),
            (LabelStage::Recover, true),
            (LabelStage::Inventory, true),
            (LabelStage::Usage, true),
            (LabelStage::Destroy, true),
        ] {
            let provider = Provider::new(DisableAt::Never);
            let harness = Harness::new(provider.backend, (), Runtime::Read);
            let mut pending = record(
                if matches!(stage, LabelStage::Usage | LabelStage::Destroy) {
                    StartPhase::Writing
                } else {
                    StartPhase::Creating
                },
            );
            if pending.phase == StartPhase::Creating {
                pending.sandbox = None;
            }
            harness.seed(pending);
            provider
                .label
                .set(stage, change, harness.database.store.clone());
            let outcome = if cleanup {
                harness.service.cleanup(id(1)).await.unwrap();
                None
            } else {
                Some(harness.service.resume(id(1), id(11)).await.unwrap())
            };
            assert_eq!(matches!(outcome, Some(StartOutcome::Ended(_))), !cleanup);
            assert_eq!(*provider.label.applied.lock().unwrap(), 1);
            assert_eq!(provider.counts.lock().unwrap().creates, 0);
            assert_eq!(provider.counts.lock().unwrap().inputs, 0);
            assert_eq!(provider.counts.lock().unwrap().launches, 0);
            assert_eq!(provider.counts.lock().unwrap().deletes, 1);
            assert_eq!(harness.start(id(11)).phase, StartPhase::Ended);
        }
    }
}

#[tokio::test]
async fn label_change_during_launch_preserves_a_confirmed_running_guest() {
    for change in [Change::BoatOff, Change::Cpu, Change::Memory, Change::Disk] {
        for stage in [LabelStage::Launch, LabelStage::LaunchInspection] {
            let provider = Provider::new(DisableAt::Never);
            *provider.launched.lock().unwrap() = Some(LaunchState::Started);
            let kind = if stage == LabelStage::Launch {
                Runtime::Start
            } else {
                Runtime::Read
            };
            let needed = RunnerAuthorityMock::retire
                .some_call(matching!(_, _, _))
                .returns(Ok(Retirement::RunningJob));
            let harness = Harness::new(provider.backend, needed, kind);
            provider
                .label
                .set(stage, change, harness.database.store.clone());
            let result = if stage == LabelStage::Launch {
                harness.service.start(id(1), request()).await.unwrap()
            } else {
                harness.seed(record(StartPhase::Launching));
                disable(&harness.database.store);
                harness.service.resume(id(1), id(11)).await.unwrap()
            };
            assert!(matches!(result, StartOutcome::Started(_)));
            assert_eq!(*provider.label.applied.lock().unwrap(), 1);
            assert!(matches!(
                harness.service.resume(id(1), id(11)).await.unwrap(),
                StartOutcome::Started(_)
            ));
            harness.service.cleanup(id(1)).await.unwrap();
            assert_eq!(
                provider.counts.lock().unwrap().launches,
                usize::from(stage == LabelStage::Launch)
            );
            assert_eq!(provider.counts.lock().unwrap().deletes, 0);
            assert_eq!(harness.start(id(11)).phase, StartPhase::Running);
        }
    }
}

#[tokio::test]
async fn largest_fallback_checks_requested_dimensions_and_preserves_unknown_create() {
    let provider = Provider::new(DisableAt::Never);
    *provider.inspect_state.lock().unwrap() = SandboxState::Creating;
    let harness = Harness::new(provider.backend, (), Runtime::Start);
    let mut label = harness.database.store.labels(id(1)).unwrap().remove(0);
    label.size.vcpus = 32;
    label.size.memory_gb = 64;
    label.size.disk_gb = 500;
    label.when_nothing_fits = WhenNothingFits::LargestBoat;
    harness.database.store.update_label(id(1), &label).unwrap();
    let mut action = request();
    action.action.requested_size.vcpu = 32;
    action.action.requested_size.memory_bytes = 64_000_000_000;
    action.action.requested_size.disk_bytes = 500_000_000_000;
    assert!(matches!(
        harness.service.start(id(1), action.clone()).await.unwrap(),
        StartOutcome::Pending(_)
    ));
    assert_eq!(provider.counts.lock().unwrap().creates, 1);
    change_label(&harness.database.store, Change::Memory);
    *provider.inspect_state.lock().unwrap() = SandboxState::Ready;
    assert!(matches!(
        harness.service.start(id(1), action).await.unwrap(),
        StartOutcome::Ended(_)
    ));
    assert_eq!(provider.counts.lock().unwrap().inputs, 0);
    assert_eq!(provider.counts.lock().unwrap().launches, 0);

    let mut ambiguous = record(StartPhase::Creating);
    ambiguous.delivery_id = id(41);
    ambiguous.runner_id = id(42);
    ambiguous.requested_vcpus = 32;
    ambiguous.requested_memory_gb = 68;
    ambiguous.requested_disk_gb = 500;
    ambiguous.sandbox = None;
    harness.seed(ambiguous);
    *provider.recovered.lock().unwrap() = false;
    change_label(&harness.database.store, Change::BoatOff);
    *harness.now.lock().unwrap() = "2026-11-07T12:00:00Z".parse().unwrap();
    assert!(matches!(
        harness.service.resume(id(1), id(41)).await.unwrap(),
        StartOutcome::Pending(_)
    ));
    assert_eq!(harness.start(id(41)).phase, StartPhase::Creating);
    assert_eq!(harness.start(id(41)).ends_at, None);
    assert_eq!(provider.counts.lock().unwrap().creates, 1);
}
