//! A forwarding PgStore decorator that captures successful service writes.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use ciproxy_overflow::crypto::KeyContext;
use ciproxy_overflow::secrets::SecretBytes;
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::overflow::admission::{
    DispatchFacts, PreparedDispatch, ReservationBudget, ReservationOutcome,
};
use ciproxy_store_interface::overflow::connection::ConnectionGrant;
use ciproxy_store_interface::overflow::record::{EncryptedLaunch, StartPhase, StartRecord};
use ciproxy_store_interface::overflow::store::{
    AccountRecord, OverflowSession, OverflowSnapshot, OverflowStore,
};
use ciproxy_store_interface::secrets::TokenHash;
use ciproxy_store_interface::store::Store;
use uuid::Uuid;

#[derive(Default)]
pub(super) struct Capture {
    pub(super) active_backends: usize,
    pub(super) key_opens: usize,
    pub(super) fail_on_key_open: Option<usize>,
    pub(super) key_open_effect: Option<(usize, Box<dyn FnOnce() + Send>)>,
    pub(super) dispatch_effect: Option<Box<dyn FnOnce() + Send>>,
    pub(super) disable_on_key_open: Option<(usize, Arc<dyn Store>)>,
    pub(super) suspend_on_key_open: Option<(usize, Arc<dyn Store>)>,
    pub(super) limit_on_launch_seal: Option<(i64, Arc<dyn Store>)>,
    pub(super) launches: BTreeMap<Uuid, EncryptedLaunch>,
    pub(super) hashes: BTreeMap<Uuid, TokenHash>,
    pub(super) sealed: BTreeMap<(Uuid, u8), SecretBytes>,
    pub(super) contexts: Vec<KeyContext>,
    pub(super) phases: Vec<StartPhase>,
    pub(super) saved: Vec<StartRecord>,
    pub(super) budgets: Vec<ReservationBudget>,
}

pub(super) struct RecordingStore {
    pub(super) inner: Arc<dyn OverflowStore>,
    pub(super) capture: Arc<Mutex<Capture>>,
}

struct RecordingSession {
    inner: Box<dyn OverflowSession>,
    capture: Arc<Mutex<Capture>>,
}

impl RecordingStore {
    fn wrap(&self, inner: Box<dyn OverflowSession>) -> Box<dyn OverflowSession> {
        Box::new(RecordingSession {
            inner,
            capture: self.capture.clone(),
        })
    }
}

impl OverflowStore for RecordingStore {
    fn try_lock_overflow(&self, workspace_id: Uuid) -> Result<Option<Box<dyn OverflowSession>>> {
        Ok(self
            .inner
            .try_lock_overflow(workspace_id)?
            .map(|inner| self.wrap(inner)))
    }

    fn lock_overflow(&self, workspace_id: Uuid) -> Result<Box<dyn OverflowSession>> {
        Ok(self.wrap(self.inner.lock_overflow(workspace_id)?))
    }

    fn overflow_token_workspace(&self, hash: &TokenHash) -> Result<Option<Uuid>> {
        self.inner.overflow_token_workspace(hash)
    }
}

impl RecordingSession {
    fn record(&self, record: &StartRecord) {
        let mut capture = self.capture.lock().unwrap();
        capture.phases.push(record.phase);
        capture.saved.push(record.clone());
    }
}

impl OverflowSession for RecordingSession {
    fn snapshot(&mut self) -> Result<OverflowSnapshot> {
        self.inner.snapshot()
    }

    fn startup_account(&mut self, delivery_id: Uuid) -> Result<Option<AccountRecord>> {
        self.inner.startup_account(delivery_id)
    }

    fn dispatch_facts(&mut self, delivery_id: Uuid) -> Result<Option<DispatchFacts>> {
        self.inner.dispatch_facts(delivery_id)
    }

    fn begin_connection(&mut self, key_id: Uuid) -> Result<Option<ConnectionGrant>> {
        self.inner.begin_connection(key_id)
    }

    fn complete_connection(
        &mut self,
        grant: ConnectionGrant,
        account: &AccountRecord,
        limit: i64,
    ) -> Result<bool> {
        self.inner.complete_connection(grant, account, limit)
    }

    fn disconnect(&mut self, at: DateTime<Utc>) -> Result<()> {
        self.inner.disconnect(at)
    }

    fn save_account(&mut self, account: &AccountRecord) -> Result<()> {
        self.inner.save_account(account)
    }

    fn save_refusal(&mut self, account: &AccountRecord, record: &StartRecord) -> Result<()> {
        self.inner.save_refusal(account, record)?;
        self.record(record);
        Ok(())
    }

    fn save_pending(
        &mut self,
        record: &StartRecord,
        launch: &EncryptedLaunch,
        hash: &TokenHash,
    ) -> Result<bool> {
        let accepted = self.inner.save_pending(record, launch, hash)?;
        if accepted {
            self.record(record);
            let mut capture = self.capture.lock().unwrap();
            capture.launches.insert(record.delivery_id, launch.clone());
            capture.hashes.insert(record.delivery_id, hash.clone());
        }
        Ok(accepted)
    }

    fn encrypted_launch(&mut self, delivery_id: Uuid) -> Result<Option<EncryptedLaunch>> {
        self.inner.encrypted_launch(delivery_id)
    }

    fn begin_dispatch(
        &mut self,
        pending: &StartRecord,
        dispatch: &PreparedDispatch,
    ) -> Result<ReservationOutcome> {
        let effect = self.capture.lock().unwrap().dispatch_effect.take();
        if let Some(effect) = effect {
            effect();
        }
        self.capture
            .lock()
            .unwrap()
            .budgets
            .push(dispatch.budget.clone());
        let outcome = self.inner.begin_dispatch(pending, dispatch)?;
        if outcome == ReservationOutcome::Reserved {
            self.record(&dispatch.apply(pending));
        }
        Ok(outcome)
    }

    fn save_start(&mut self, record: &StartRecord) -> Result<()> {
        self.inner.save_start(record)?;
        self.record(record);
        Ok(())
    }

    fn accept_end(
        &mut self,
        hash: &TokenHash,
        runner: Uuid,
        at: DateTime<Utc>,
    ) -> Result<Option<StartRecord>> {
        self.inner.accept_end(hash, runner, at)
    }

    fn collect(&mut self, before: DateTime<Utc>) -> Result<()> {
        self.inner.collect(before)
    }
}
