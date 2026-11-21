//! Accounting requests coalesce, cancellation drops work, and failures stay local.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use ciproxy_control_plane::accounting::{Accounting, AccountingWorker};
use ciproxy_github::timing::{JobTiming, JobTimingMock};
use ciproxy_metering::service::StoredMetering;
use ciproxy_store_interface::workspaces::Workspace;
use tokio::sync::{Semaphore, watch};
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use super::clock::clock;
use super::recovery_support::job;
use super::support::{Fixture, at};

struct TimingGate {
    calls: AtomicUsize,
    active: AtomicUsize,
    entered: Semaphore,
    release: Semaphore,
    dropped: Semaphore,
}

struct Active(Arc<TimingGate>);

impl Drop for Active {
    fn drop(&mut self) {
        self.0.active.fetch_sub(1, Ordering::SeqCst);
        self.0.dropped.add_permits(1);
    }
}

struct GatedTiming(Arc<TimingGate>);

#[async_trait]
impl JobTiming for GatedTiming {
    async fn repair(&self, _: Uuid) -> ciproxy_github::error::Result<()> {
        self.0.calls.fetch_add(1, Ordering::SeqCst);
        self.0.active.fetch_add(1, Ordering::SeqCst);
        let _active = Active(self.0.clone());
        self.0.entered.add_permits(1);
        self.0.release.acquire().await.unwrap().forget();
        Ok(())
    }
}

#[tokio::test]
async fn accounting_coalesces_requests_and_cancels_pending_work_on_stop_or_abort() {
    let fixture = Fixture::new();
    let gate = Arc::new(TimingGate {
        calls: AtomicUsize::new(0),
        active: AtomicUsize::new(0),
        entered: Semaphore::new(0),
        release: Semaphore::new(0),
        dropped: Semaphore::new(0),
    });
    let worker = Arc::new(AccountingWorker::new(
        fixture.store.clone(),
        Arc::new(GatedTiming(gate.clone())),
        Arc::new(StoredMetering::new(fixture.store.clone())),
        clock(false),
    ));
    let outcome = tokio::time::timeout(Duration::from_secs(10), async {
        let (stop, receiver) = watch::channel(false);
        let running = worker.clone();
        let task = tokio::spawn(async move { running.run(receiver).await });
        worker.request();
        gate.entered.acquire().await.unwrap().forget();
        for _ in 0..20 {
            worker.request();
        }
        assert_eq!(gate.calls.load(Ordering::SeqCst), 1);
        assert_eq!(gate.active.load(Ordering::SeqCst), 1);
        gate.release.add_permits(1);
        gate.dropped.acquire().await.unwrap().forget();
        gate.entered.acquire().await.unwrap().forget();
        assert_eq!(gate.calls.load(Ordering::SeqCst), 2);
        assert_eq!(gate.active.load(Ordering::SeqCst), 1);
        stop.send_replace(true);
        task.await.unwrap();
        gate.dropped.acquire().await.unwrap().forget();
        assert_eq!(gate.active.load(Ordering::SeqCst), 0);
        let (_stop, receiver) = watch::channel(false);
        let running = worker.clone();
        let task = tokio::spawn(async move { running.run(receiver).await });
        worker.request();
        gate.entered.acquire().await.unwrap().forget();
        assert_eq!(gate.calls.load(Ordering::SeqCst), 3);
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
        gate.dropped.acquire().await.unwrap().forget();
        assert_eq!(gate.active.load(Ordering::SeqCst), 0);
    })
    .await;
    assert_eq!(
        outcome,
        Ok(()),
        "accounting cancellation did not release pending work"
    );
}

#[tokio::test]
async fn timing_panic_or_error_keeps_independent_workspace_accounting_running() {
    for panics in [true, false] {
        let fixture = Fixture::new();
        let failed = Uuid::nil();
        fixture
            .store
            .create_workspace(&Workspace {
                id: failed,
                name: "Accounting failure".to_owned(),
                delete_after: None,
            })
            .unwrap();
        fixture
            .store
            .save_github_job(fixture.workspace, &job(&fixture), at())
            .unwrap();
        let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
        let observed = calls.clone();
        let timing = Arc::new(Unimock::new(
            JobTimingMock::repair
                .each_call(matching!(_))
                .answers_arc(Arc::new(move |_, workspace| {
                    observed.lock().unwrap().push(workspace);
                    if workspace == failed {
                        if panics {
                            panic!("controlled workspace timing panic");
                        }
                        return Err(ciproxy_github::error::Error::OrganizationInactive);
                    }
                    Ok(())
                })),
        ));
        let worker = AccountingWorker::new(
            fixture.store.clone(),
            timing,
            Arc::new(StoredMetering::new(fixture.store.clone())),
            clock(false),
        );
        worker.pass().await;
        assert_eq!(*calls.lock().unwrap(), vec![failed, fixture.workspace]);
        assert_eq!(
            fixture
                .store
                .workspace_usage(fixture.workspace)
                .unwrap()
                .accounted_jobs
                .len(),
            1
        );
    }
}
