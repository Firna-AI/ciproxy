//! Guest exit usage and exit reports synthesized from VM liveness.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::guest::GuestToAgent;
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::runners::RunnerExit;
use ciproxy_protocol::usage::{JobStep, JobStepTiming, JobUsage};
use tokio::sync::{Mutex, mpsc, watch};
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{CacheMock, GuestProvenance, StoreMock, VmsMock};
use crate::controller::Agent;
use crate::error::Error;
use crate::fixtures::{controller, vm};
use crate::state::MachineState;
use crate::vm_liveness::{UnitState, VmLivenessMock};
use crate::vsock::Configuration;
use crate::vsock_stream::GuestBridge;

fn job_usage() -> JobUsage {
    JobUsage {
        platform: Platform::LinuxX64,
        first_point_at_ms: 1_750_000_000_000,
        point_seconds: 1,
        cpu_milli_vcpu: vec![750, 1_250],
        peak_memory_mib: vec![1_024, 1_536],
        oom_kills: Some(vec![0, 1]),
        steps: vec![JobStep {
            name: "Build".into(),
            timing: JobStepTiming::Ran {
                started_at_ms: Some(1_750_000_000_000),
                ended_at_ms: Some(1_750_000_002_000),
            },
        }],
    }
}

#[tokio::test]
async fn guest_exit_forwards_job_usage() {
    let record = vm("runner", true);
    let runner_id = record.report.runner_id.clone();
    let agent = controller(
        MachineState {
            runners: vec![record],
            ..MachineState::default()
        },
        Unimock::new((
            VmsMock::stop.some_call(matching!(_)).returns(Ok(())),
            CacheMock::release.some_call(matching!(_)).returns(Ok(())),
            StoreMock::save.some_call(matching!(_)).returns(Ok(())),
        )),
    );
    let exit = RunnerExit { code: Some(1) };
    let usage = job_usage();
    let response = agent
        .guest(
            GuestProvenance::Recovery,
            &runner_id.0,
            GuestToAgent::RunnerExited {
                runner_id: runner_id.clone(),
                exit,
                usage: Some(usage.clone()),
            },
        )
        .await
        .unwrap();
    assert_eq!(
        response,
        Some(AgentMessage::RunnerExited {
            runner_id,
            exit,
            usage: Some(usage),
        })
    );
}

#[tokio::test]
async fn agent_generated_exit_reports_have_no_job_usage() {
    for (state, code) in [(UnitState::Exited(7), Some(7)), (UnitState::Absent, None)] {
        let record = vm("runner", true);
        let runner_id = record.report.runner_id.clone();
        let mock = Arc::new(Unimock::new(
            VmLivenessMock::state
                .some_call(matching!(_))
                .returns(Ok(state)),
        ));
        let (events, mut event_receiver) = mpsc::channel(1);
        let (_commands, command_receiver) = mpsc::channel(1);
        let (_shutdown, shutdown_receiver) = watch::channel(false);
        let bridge = GuestBridge {
            configuration: Arc::new(Mutex::new(Configuration::new(None))),
            transport: mock.clone(),
            liveness: mock,
            events,
            vm: record,
            initial: AtomicBool::new(false),
        };

        bridge.run(command_receiver, shutdown_receiver).await;

        assert_eq!(
            event_receiver.try_recv().unwrap().message,
            GuestToAgent::RunnerExited {
                runner_id,
                exit: RunnerExit { code },
                usage: None,
            },
            "VM state: {state:?}"
        );
    }
}

#[tokio::test]
async fn rejects_boat_usage_samples_over_vsock() {
    let record = vm("runner", true);
    let runner_id = record.report.runner_id.clone();
    let agent = controller(
        MachineState {
            runners: vec![record],
            ..MachineState::default()
        },
        Unimock::new(()),
    );
    let response = agent
        .guest(
            GuestProvenance::Recovery,
            &runner_id.0,
            GuestToAgent::UsageSamples {
                runner_id: runner_id.clone(),
                usage: job_usage(),
            },
        )
        .await;
    assert!(
        matches!(response, Err(Error::GuestMismatch)),
        "{response:?}"
    );
}
