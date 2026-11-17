//! Cache reports replay the latest budget after a service restart.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use ciproxy_protocol::cache::MachineCacheReport;
use ciproxy_protocol::jobs::JobFacts;
use ciproxy_protocol::runtime::CacheRegistration;
use futures_util::{pin_mut, poll};
use tokio::sync::oneshot;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::Cache;
use crate::cache::HttpCache;
use crate::cache_admin::{Administration, AdministrationMock, CacheFilesMock, Credential};
use crate::error::{Error, Result};
use crate::fixtures::empty_cache;
use crate::process::{CommandsMock, UnitState};

struct WaitingAdministration {
    delegate: Arc<dyn Administration>,
    release: tokio::sync::Mutex<Option<oneshot::Receiver<()>>>,
}

#[async_trait]
impl Administration for WaitingAdministration {
    async fn recover(&self, registration: &CacheRegistration) -> Result<Option<Credential>> {
        self.delegate.recover(registration).await
    }
    async fn fallback(&self, facts: &JobFacts) -> Result<()> {
        self.delegate.fallback(facts).await
    }
    async fn budget(&self, bytes: u64) -> Result<()> {
        if bytes == 20
            && let Some(release) = self.release.lock().await.take()
        {
            release.await.unwrap();
        }
        self.delegate.budget(bytes).await
    }
    async fn report(&self) -> Result<MachineCacheReport> {
        self.delegate.report().await
    }
    async fn clear(&self) -> Result<()> {
        self.delegate.clear().await
    }
    async fn register(&self, registration: CacheRegistration) -> Result<Credential> {
        self.delegate.register(registration).await
    }
    async fn revoke(&self, facts: &JobFacts) -> Result<()> {
        self.delegate.revoke(facts).await
    }
}

#[tokio::test]
async fn report_waits_for_a_pending_budget_reduction_before_replaying_it() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let budgets = events.clone();
    let reports = events.clone();
    let mock = Arc::new(Unimock::new((
        AdministrationMock::budget
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, bytes| {
                budgets.lock().unwrap().push(format!("budget:{bytes}"));
                Ok(())
            })),
        AdministrationMock::report
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                reports.lock().unwrap().push("report".into());
                Ok(empty_cache())
            })),
    )));
    let (release, wait) = oneshot::channel();
    let administration = Arc::new(WaitingAdministration {
        delegate: mock.clone(),
        release: tokio::sync::Mutex::new(Some(wait)),
    });
    let cache = HttpCache::with_boundaries(administration, mock.clone(), mock);
    cache.refresh_budget(80).await.unwrap();
    let reduce = cache.budget(20);
    pin_mut!(reduce);
    assert!(poll!(reduce.as_mut()).is_pending());
    let report = cache.report();
    pin_mut!(report);
    assert!(poll!(report.as_mut()).is_pending());
    release.send(()).unwrap();
    reduce.await.unwrap();
    report.await.unwrap();
    assert_eq!(
        *events.lock().unwrap(),
        vec!["budget:80", "budget:20", "budget:20", "report"]
    );
}

#[tokio::test]
async fn report_replays_the_latest_reduced_budget_after_service_restart() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let budgets = events.clone();
    let commands = events.clone();
    let reads = events.clone();
    let reclaimed = events.clone();
    let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mock = Arc::new(Unimock::new((
        AdministrationMock::budget
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, bytes| {
                budgets.lock().unwrap().push(format!("budget:{bytes}"));
                if calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 1 {
                    Err(Error::CacheUnavailable)
                } else {
                    Ok(())
                }
            })),
        CommandsMock::unit
            .some_call(matching!(_))
            .returns(Ok(UnitState::Active)),
        CommandsMock::run
            .each_call(matching!(_, _, _))
            .answers_arc(Arc::new(move |_, _, args, _| {
                commands.lock().unwrap().push(args[0].clone());
                Ok(Vec::new())
            })),
        CacheFilesMock::release_occupied_space
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                reclaimed.lock().unwrap().push("reclaim".into());
                Ok(())
            })),
        AdministrationMock::report
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                reads.lock().unwrap().push("report".into());
                Ok(empty_cache())
            })),
    )));
    let cache = HttpCache::with_boundaries(mock.clone(), mock.clone(), mock);
    cache.refresh_budget(80).await.unwrap();
    cache.budget(20).await.unwrap();
    cache.report().await.unwrap();
    assert_eq!(
        *events.lock().unwrap(),
        vec![
            "budget:80",
            "budget:20",
            "stop",
            "reclaim",
            "start",
            "budget:20",
            "report",
        ]
    );
}
