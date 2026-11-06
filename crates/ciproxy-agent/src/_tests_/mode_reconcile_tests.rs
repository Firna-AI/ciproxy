//! Delayed collaborators cannot restore an old mode after credential removal.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::cache::{CacheConnection, MachineCacheReport};
use ciproxy_protocol::identifiers::{JobId, RunnerId};
use ciproxy_protocol::jobs::JobFacts;
use ciproxy_protocol::jobs::JobScope;
use ciproxy_protocol::runtime::RuntimeCredentials;
use futures_util::{pin_mut, poll};
use tokio::sync::{Mutex, oneshot};
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{Cache, CacheMock, HostMock, StoreMock};
use crate::controller::Agent;
use crate::error::Result;
use crate::fixtures::{controller, hardware};
use crate::state::{MachineState, Mode, VmRecord};

struct WaitingCache {
    delegate: Arc<dyn Cache>,
    barrier: Mutex<Option<oneshot::Receiver<()>>>,
}

#[async_trait]
impl Cache for WaitingCache {
    async fn fallback(&self, vm: &VmRecord) -> Result<()> {
        self.delegate.fallback(vm).await
    }
    async fn handoff(
        &self,
        vm: &VmRecord,
        job_id: &JobId,
        runtime: RuntimeCredentials,
    ) -> Result<()> {
        self.delegate.handoff(vm, job_id, runtime).await
    }
    async fn prepare(
        &self,
        runner: &RunnerId,
        facts: &JobFacts,
    ) -> Result<Option<(CacheConnection, JobScope, u64)>> {
        self.delegate.prepare(runner, facts).await
    }
    async fn budget(&self, bytes: u64) -> Result<()> {
        let barrier = self.barrier.lock().await.take().unwrap();
        barrier.await.unwrap();
        self.delegate.budget(bytes).await
    }
    async fn report(&self) -> Result<MachineCacheReport> {
        self.delegate.report().await
    }
    async fn clear(&self) -> Result<()> {
        self.delegate.clear().await
    }
    async fn release(&self, vm: &VmRecord) -> Result<()> {
        self.delegate.release(vm).await
    }
}

#[tokio::test]
async fn removal_during_a_delayed_reconciliation_stays_terminal() {
    let mock = Unimock::new((
        HostMock::hardware
            .some_call(matching!())
            .returns(Ok(hardware())),
        CacheMock::budget
            .some_call(matching!(645_000_000_000))
            .returns(Ok(())),
        StoreMock::save
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
    ));
    let (release, barrier) = oneshot::channel();
    let mut agent = controller(
        MachineState {
            mode: Mode::Draining,
            ..MachineState::default()
        },
        mock.clone(),
    );
    agent.cache = Arc::new(WaitingCache {
        delegate: Arc::new(mock.clone()),
        barrier: Mutex::new(Some(barrier)),
    });
    let reconcile = agent.reconcile();
    pin_mut!(reconcile);
    assert!(poll!(reconcile.as_mut()).is_pending());
    agent.remove().await.unwrap();
    release.send(()).unwrap();
    reconcile.await.unwrap();
    assert_eq!(agent.state.lock().await.mode, Mode::Removed);
    assert!(agent.finished().await);
}
