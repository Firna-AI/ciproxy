//! Real registration persistence with controlled remote runner answers.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ciproxy_control_plane::runtime::MachineRuntime;
use ciproxy_gateway::gateway::MachineGateway;
use ciproxy_github::error::{Error, Result};
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::registrations::RegistrationService;
use ciproxy_github::runtime::Runtime;
use ciproxy_github::scale_sets::{JitRegistration, MessageQueue, ScaleSetApi, ScaleSetFactory};
use ciproxy_store_interface::organizations::{AppState, Organization};
use uuid::Uuid;

use super::recovery_support::runtime_with_clock;
use super::support::{Fixture, at};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum Call {
    Find,
    Remove(i64),
}

#[derive(Clone, Copy)]
pub(super) enum Answer {
    Busy,
    Removed,
    Absent,
    LookupUnavailable,
    DeleteUnavailable,
}

pub(super) struct Remote {
    pub(super) calls: Mutex<Vec<Call>>,
    pub(super) contexts: Mutex<Vec<(i64, AppState)>>,
    pub(super) answer: Mutex<Answer>,
    pub(super) answers: Mutex<HashMap<i64, Answer>>,
}

impl Remote {
    pub(super) fn new(answer: Answer) -> Arc<Self> {
        Arc::new(Self {
            calls: Mutex::new(Vec::new()),
            contexts: Mutex::new(Vec::new()),
            answer: Mutex::new(answer),
            answers: Mutex::new(HashMap::new()),
        })
    }
}

pub(super) struct Factory(pub(super) Arc<Remote>);

#[async_trait]
impl ScaleSetFactory for Factory {
    async fn for_organization(&self, organization: &Organization) -> Result<Arc<dyn ScaleSetApi>> {
        self.0
            .contexts
            .lock()
            .unwrap()
            .push((organization.installation_id, organization.app_state));
        Ok(self.0.clone())
    }
}

#[async_trait]
impl ScaleSetApi for Remote {
    async fn find_runner(&self, _: &str) -> Result<Option<i64>> {
        self.calls.lock().unwrap().push(Call::Find);
        match *self.answer.lock().unwrap() {
            Answer::Absent => Ok(None),
            Answer::LookupUnavailable => Err(Error::AccessUnavailable(404)),
            _ => Ok(Some(47)),
        }
    }
    async fn remove_runner(&self, id: i64) -> Result<()> {
        self.calls.lock().unwrap().push(Call::Remove(id));
        let answer = self
            .answers
            .lock()
            .unwrap()
            .get(&id)
            .copied()
            .unwrap_or(*self.answer.lock().unwrap());
        match answer {
            Answer::Busy => Err(Error::RunnerBusy),
            Answer::DeleteUnavailable => Err(Error::AccessUnavailable(403)),
            _ => Ok(()),
        }
    }
    async fn ensure(&self, _: &str, _: i64, _: Option<i64>) -> Result<i64> {
        Err(Error::OrganizationInactive)
    }
    async fn open(&self, _: i64, _: &str) -> Result<Box<dyn MessageQueue>> {
        Err(Error::OrganizationInactive)
    }
    async fn register(&self, _: i64, _: &str) -> Result<JitRegistration> {
        Err(Error::OrganizationInactive)
    }
    async fn remove_scale_set(&self, _: i64) -> Result<()> {
        Err(Error::OrganizationInactive)
    }
    async fn rename_scale_set(&self, _: i64, _: &str) -> Result<()> {
        Err(Error::OrganizationInactive)
    }
}

pub(super) fn real_runtime(
    fixture: &Fixture,
    remote: Arc<Remote>,
    gateway: Arc<dyn MachineGateway>,
) -> MachineRuntime {
    runtime_with_clock(
        fixture,
        Arc::new(RegistrationService {
            store: Arc::new(StoreAdapter::new(
                fixture.store.clone(),
                Arc::new(unimock::Unimock::new(())),
            )),
            scale_sets: Arc::new(Factory(remote)),
        }),
        gateway,
        Arc::new(FixedRuntime),
    )
}

struct FixedRuntime;

impl Runtime for FixedRuntime {
    fn now(&self) -> DateTime<Utc> {
        at()
    }
    fn uuid(&self) -> Uuid {
        Uuid::new_v4()
    }
}
