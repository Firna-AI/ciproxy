//! A first report cannot restore a budget while root admission releases space.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use ciproxy_protocol::agent::ControlMessage;
use ciproxy_protocol::identifiers::CommandId;
use futures_util::{pin_mut, poll};
use tokio::sync::oneshot;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{CacheMock, StoreMock, VmsMock};
use crate::controller::Agent;
use crate::docker::DockerDisks;
use crate::error::Result;
use crate::fixtures::{applied, controller, report_dependencies, spec, start_image, vm};
use crate::state::MachineState;
use crate::workload_space::WorkloadSpace;

struct WaitingRelease {
    delegate: Arc<dyn WorkloadSpace>,
    wait: tokio::sync::Mutex<Option<oneshot::Receiver<()>>>,
}

#[async_trait]
impl WorkloadSpace for WaitingRelease {
    async fn available(&self, image_budget: u64) -> Result<u64> {
        self.delegate.available(image_budget).await
    }
    fn cache_budget(&self, free: u64, dockers: Option<&dyn DockerDisks>) -> Result<u64> {
        self.delegate.cache_budget(free, dockers)
    }
    async fn release(&self, free: u64, dockers: Option<&dyn DockerDisks>) -> Result<()> {
        let wait = self.wait.lock().await.take().unwrap();
        wait.await.unwrap();
        self.delegate.release(free, dockers).await
    }
}

#[tokio::test]
async fn first_report_waits_for_root_reservation_and_cache_release() {
    let budgets = Arc::new(Mutex::new(Vec::new()));
    let grants = budgets.clone();
    let reports = budgets.clone();
    let mock = Unimock::new((
        report_dependencies(),
        start_image(),
        StoreMock::save
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        VmsMock::allocate
            .some_call(matching!(_, _))
            .answers(&|_, report, _| {
                Ok(crate::state::VmRecord {
                    report,
                    ..vm("fresh", false)
                })
            }),
        VmsMock::start.some_call(matching!(_, _)).returns(Ok(())),
        CacheMock::budget
            .some_call(matching!(_))
            .answers_arc(Arc::new(move |_, bytes| {
                grants.lock().unwrap().push(("grant", bytes));
                Ok(())
            })),
        CacheMock::refresh_budget
            .some_call(matching!(_))
            .answers_arc(Arc::new(move |_, bytes| {
                reports.lock().unwrap().push(("report", bytes));
                Ok(())
            })),
    ));
    let mut agent = controller(MachineState::default(), mock);
    agent.reporting_initialized = tokio::sync::Mutex::new(false);
    let (release, wait) = oneshot::channel();
    agent.space = Arc::new(WaitingRelease {
        delegate: agent.space.clone(),
        wait: tokio::sync::Mutex::new(Some(wait)),
    });
    let start = agent.control(ControlMessage::StartRunner {
        command_id: CommandId("command-1".to_owned()),
        runner: spec(),
    });
    pin_mut!(start);
    assert!(poll!(start.as_mut()).is_pending());
    let report = agent.report();
    pin_mut!(report);
    assert!(
        poll!(report.as_mut()).is_pending(),
        "first report must wait for the allocation release"
    );
    assert_eq!(*budgets.lock().unwrap(), vec![]);
    release.send(()).unwrap();
    assert_eq!(start.await.unwrap(), applied());
    assert_eq!(report.await.unwrap().runners.len(), 1);
    assert_eq!(
        *budgets.lock().unwrap(),
        vec![("grant", 564_100_387_328), ("report", 564_100_387_328)]
    );
}
