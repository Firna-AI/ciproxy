//! Explicit barriers for remote registration progress and stale-result tests.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use ciproxy_github::error::Result;
use ciproxy_github::persistence::session::ListenerTarget;
use ciproxy_github::registrations::{
    RegisteredRunner, RegistrationRequest, RegistrationRetirement, RunnerRegistrations,
};
use tokio::sync::{Notify, Semaphore};
use uuid::Uuid;

pub(super) struct HeldRemote {
    pub(super) workspace: Uuid,
    pub(super) entered: Notify,
    pub(super) release: Semaphore,
    pub(super) released: AtomicBool,
    pub(super) fail: AtomicBool,
    pub(super) creates: AtomicUsize,
    pub(super) retirements: Mutex<Vec<Option<i64>>>,
    pub(super) hold_create: bool,
    pub(super) answer: RegistrationRetirement,
}

impl HeldRemote {
    pub(super) fn new(
        workspace: Uuid,
        hold_create: bool,
        answer: RegistrationRetirement,
    ) -> Arc<Self> {
        Arc::new(Self {
            workspace,
            entered: Notify::new(),
            release: Semaphore::new(0),
            released: AtomicBool::new(false),
            fail: AtomicBool::new(false),
            creates: AtomicUsize::new(0),
            retirements: Mutex::new(Vec::new()),
            hold_create,
            answer,
        })
    }

    async fn wait(&self) {
        self.entered.notify_one();
        self.release.acquire().await.unwrap().forget();
        self.released.store(true, Ordering::SeqCst);
    }
}

#[async_trait]
impl RunnerRegistrations for HeldRemote {
    async fn create(&self, request: RegistrationRequest, _: &str) -> Result<RegisteredRunner> {
        self.creates.fetch_add(1, Ordering::SeqCst);
        if self.hold_create && request.workspace_id == self.workspace {
            self.wait().await;
        }
        Ok(RegisteredRunner {
            target: request.target,
            runner_id: if request.workspace_id == self.workspace {
                100
            } else {
                101
            },
            configuration: "fixture-jit".into(),
        })
    }

    async fn retire(
        &self,
        target: ListenerTarget,
        _: &str,
        id: Option<i64>,
    ) -> Result<RegistrationRetirement> {
        self.retirements.lock().unwrap().push(id);
        if !self.hold_create && target.workspace_id == self.workspace {
            self.wait().await;
        }
        if self.fail.load(Ordering::SeqCst) {
            return Err(ciproxy_github::error::Error::AccessUnavailable(503));
        }
        Ok(self.answer)
    }
}
