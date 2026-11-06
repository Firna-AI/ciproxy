//! A deterministic barrier holds VM startup while hardware reports continue.

use std::sync::Arc;
use std::task::Poll;

use async_trait::async_trait;
use ciproxy_protocol::agent::ControlMessage;
use ciproxy_protocol::guest::AgentToGuest;
use ciproxy_protocol::identifiers::CommandId;
use ciproxy_protocol::runners::RunnerReport;
use futures_util::{pin_mut, poll};
use tokio::sync::{Mutex, oneshot};
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{CacheMock, StoreMock, Vms, VmsMock};
use crate::controller::Agent;
use crate::error::Result;
use crate::fixtures::{applied, controller, report_dependencies, spec, start_image, vm};
use crate::state::{MachineState, VmRecord};

struct PausedVms {
    delegate: Arc<dyn Vms>,
    barrier: Mutex<Option<oneshot::Receiver<()>>>,
}

#[async_trait]
impl Vms for PausedVms {
    async fn configuration_lost(&self, vm: &VmRecord) -> Result<bool> {
        self.delegate.configuration_lost(vm).await
    }
    async fn clear_docker(&self, vm: &VmRecord) -> Result<()> {
        self.delegate.clear_docker(vm).await
    }
    async fn swap_docker(&self, vm: &VmRecord) -> Result<()> {
        self.delegate.swap_docker(vm).await
    }
    fn allocate(&self, report: RunnerReport, used: &[VmRecord]) -> Result<VmRecord> {
        self.delegate.allocate(report, used)
    }
    async fn start(&self, vm: &VmRecord, config: AgentToGuest) -> Result<()> {
        let barrier = self.barrier.lock().await.take().unwrap();
        barrier.await.unwrap();
        self.delegate.start(vm, config).await
    }
    async fn restore(&self, vm: &VmRecord) -> Result<()> {
        self.delegate.restore(vm).await
    }
    async fn request_stop(&self, vm: &VmRecord) -> Result<()> {
        self.delegate.request_stop(vm).await
    }
    async fn stop(&self, vm: &VmRecord) -> Result<()> {
        self.delegate.stop(vm).await
    }
    async fn send(&self, vm: &VmRecord, message: &AgentToGuest) -> Result<()> {
        self.delegate.send(vm, message).await
    }
}

#[tokio::test]
async fn hardware_reports_continue_while_vm_startup_is_pending() {
    let mock = Unimock::new((
        report_dependencies(),
        start_image(),
        StoreMock::save.some_call(matching!(_)).returns(Ok(())),
        VmsMock::allocate
            .some_call(matching!(_, _))
            .answers(&|_, report, _| {
                Ok(VmRecord {
                    cache_permissions: None,
                    job_facts: None,
                    report,
                    ..vm("runner-1", false)
                })
            }),
        CacheMock::budget
            .some_call(matching!(564_100_387_328))
            .returns(Ok(())),
        VmsMock::start.some_call(matching!(_, _)).returns(Ok(())),
    ));
    let (release, barrier) = oneshot::channel();
    let mut agent = controller(MachineState::default(), mock.clone());
    agent.vms = Arc::new(PausedVms {
        delegate: Arc::new(mock.clone()),
        barrier: Mutex::new(Some(barrier)),
    });
    let command = agent.control(ControlMessage::StartRunner {
        command_id: CommandId("command-1".to_owned()),
        runner: spec(),
    });
    pin_mut!(command);
    assert!(poll!(command.as_mut()).is_pending());
    let report = agent.report();
    pin_mut!(report);
    let observed = match poll!(report.as_mut()) {
        Poll::Ready(Ok(report)) => report,
        other => panic!("report must be ready while startup waits: {other:?}"),
    };
    assert_eq!(observed.runners.len(), 1);
    assert_eq!(observed.capacity.0, 6);
    release.send(()).unwrap();
    assert_eq!(command.await.unwrap(), applied());
}
