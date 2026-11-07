//! Controlled external provider, encryption, clock, and registration facts.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ciproxy_github::runtime::Runtime;
use ciproxy_overflow::clock::Clock;
use ciproxy_overflow::crypto::{KeyCipher, KeyContext};
use ciproxy_overflow::error::{Error, Result};
use ciproxy_overflow::maintenance::MaintenanceCapabilities;
use ciproxy_overflow::provider::{
    AccountFacts, BackendFactory, BoatAccountApiMock, Capabilities, GuestLauncherMock, RuntimeUsage,
};
use ciproxy_overflow::secrets::{SecretBytes, TokenSource};
use ciproxy_protocol::secret::Secret;
use rust_decimal::Decimal;
use sandbox_interface::{
    BackendCreateSandboxRequest, BackendSandbox, ProviderRef, SandboxAccountCapacityMock,
    SandboxFilesMock, SandboxLifecycleMock, SandboxState,
};
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

pub(super) struct Ports {
    pub(super) usage_usd: Mutex<Option<Decimal>>,
    pub(super) now: Mutex<DateTime<Utc>>,
    pub(super) fail: AtomicBool,
    pub(super) reject: AtomicBool,
    pub(super) trial: AtomicBool,
    pub(super) pending: AtomicBool,
    pub(super) delete_fail: AtomicBool,
    pub(super) creates: Mutex<Vec<BackendCreateSandboxRequest>>,
    pub(super) launches: AtomicUsize,
    pub(super) deletes: AtomicUsize,
    pub(super) inputs: Mutex<Vec<serde_json::Value>>,
    ids: AtomicU64,
    sealed: Mutex<HashMap<Vec<u8>, SecretBytes>>,
}

impl Ports {
    pub(super) fn new() -> Arc<Self> {
        Arc::new(Self {
            usage_usd: Mutex::new(None),
            now: Mutex::new("2026-11-05T12:00:00Z".parse().unwrap()),
            fail: AtomicBool::new(false),
            reject: AtomicBool::new(false),
            trial: AtomicBool::new(false),
            pending: AtomicBool::new(false),
            delete_fail: AtomicBool::new(false),
            creates: Mutex::new(Vec::new()),
            launches: AtomicUsize::new(0),
            deletes: AtomicUsize::new(0),
            inputs: Mutex::new(Vec::new()),
            ids: AtomicU64::new(1000),
            sealed: Mutex::new(HashMap::new()),
        })
    }
    pub(super) fn backend(self: &Arc<Self>) -> Capabilities {
        let account = self.clone();
        let create = self.clone();
        let files = self.clone();
        let launch = self.clone();
        let delete = self.clone();
        let usage = self.clone();
        let mock = Arc::new(
            Unimock::new((
                BoatAccountApiMock::account
                    .some_call(matching!())
                    .answers_arc(Arc::new(move |_| {
                        if account.reject.load(Ordering::SeqCst) {
                            return Err(Error::RejectedKey);
                        }
                        Ok(AccountFacts {
                            name: "Customer Boat".to_owned(),
                            wallet: "personal".to_owned(),
                            trial: account.trial.load(Ordering::SeqCst),
                            billing_ready: true,
                            start_fee_bound_usd: Some(Decimal::new(7, 2)),
                        })
                    })),
                SandboxAccountCapacityMock::account_capacity
                    .some_call(matching!())
                    .answers(&|_| Ok(vec![sandbox_boat::catalogue::offer("large").unwrap()])),
                SandboxLifecycleMock::create_sandbox
                    .some_call(matching!(_))
                    .answers_arc(Arc::new(move |_, request| {
                        let id = request.sandbox_id.as_uuid();
                        create.creates.lock().unwrap().push(request);
                        if create.pending.load(Ordering::SeqCst) {
                            return Err(sandbox_interface::Error::BackendUnavailable {
                                backend_id: "boat".to_owned(),
                            });
                        }
                        Ok(BackendSandbox {
                            provider_ref: ProviderRef::new(id.to_string()),
                            state: SandboxState::Ready,
                        })
                    })),
                SandboxLifecycleMock::recover_sandbox_create
                    .some_call(matching!(_))
                    .returns(Ok(None)),
                SandboxLifecycleMock::list_managed_sandboxes
                    .some_call(matching!(_))
                    .returns(Ok(Vec::new())),
                SandboxLifecycleMock::inspect_sandbox
                    .some_call(matching!(_))
                    .answers(&|_, provider| {
                        Ok(BackendSandbox {
                            provider_ref: provider,
                            state: SandboxState::Ready,
                        })
                    }),
                SandboxLifecycleMock::destroy_sandbox
                    .some_call(matching!(_))
                    .answers_arc(Arc::new(move |_, _| {
                        delete.deletes.fetch_add(1, Ordering::SeqCst);
                        if delete.delete_fail.load(Ordering::SeqCst) {
                            return Err(sandbox_interface::Error::BackendUnavailable {
                                backend_id: "boat".to_owned(),
                            });
                        }
                        Ok(())
                    })),
                BoatAccountApiMock::usage
                    .some_call(matching!(_, _, _))
                    .answers_arc(Arc::new(move |_, _, _, until| {
                        Ok(usage.usage_usd.lock().unwrap().map(|usd| RuntimeUsage {
                            since: *usage.now.lock().unwrap() - chrono::Duration::seconds(1),
                            until,
                            usd,
                            running: false,
                        }))
                    })),
                SandboxFilesMock::write_file
                    .some_call(matching!(_))
                    .answers_arc(Arc::new(move |_, input| {
                        if !input.bytes.is_empty() {
                            files
                                .inputs
                                .lock()
                                .unwrap()
                                .push(serde_json::from_slice(&input.bytes).unwrap());
                        }
                        Ok(())
                    })),
                GuestLauncherMock::prepare
                    .some_call(matching!(_, _))
                    .returns(Ok(())),
                GuestLauncherMock::launch
                    .some_call(matching!(_, _))
                    .answers_arc(Arc::new(move |_, _, _| {
                        launch.launches.fetch_add(1, Ordering::SeqCst);
                        Ok(())
                    })),
            ))
            .no_verify_in_drop(),
        );
        Capabilities {
            lifecycle: mock.clone(),
            capacity: mock.clone(),
            files: mock.clone(),
            account: mock.clone(),
            launcher: mock.clone(),
            inspection: mock,
        }
    }
}

pub(super) struct Factory(pub(super) Arc<Ports>);
impl BackendFactory for Factory {
    fn backend(&self, _: &Secret, _: &str) -> Result<Capabilities> {
        Ok(self.0.backend())
    }
    fn maintenance(&self, _: &Secret, _: &str) -> Result<MaintenanceCapabilities> {
        Ok(self.0.backend().maintenance())
    }
}

impl Clock for Ports {
    fn now(&self) -> DateTime<Utc> {
        *self.now.lock().unwrap()
    }
}
impl Runtime for Ports {
    fn now(&self) -> DateTime<Utc> {
        Clock::now(self)
    }
    fn uuid(&self) -> Uuid {
        Uuid::from_u128(u128::from(self.ids.fetch_add(1, Ordering::SeqCst)))
    }
}
impl TokenSource for Ports {
    fn token(&self) -> Secret {
        Secret::new(format!("fake-callback-{}", Runtime::uuid(self)))
    }
}
#[async_trait]
impl KeyCipher for Ports {
    async fn seal(&self, _: KeyContext, bytes: SecretBytes) -> Result<Vec<u8>> {
        let id = Runtime::uuid(self).as_bytes().to_vec();
        self.sealed.lock().unwrap().insert(id.clone(), bytes);
        Ok(id)
    }
    async fn open(&self, _: KeyContext, bytes: Vec<u8>) -> Result<SecretBytes> {
        if self.fail.load(Ordering::SeqCst) {
            return Err(Error::ProviderUnavailable);
        }
        Ok(self.sealed.lock().unwrap().get(&bytes).unwrap().clone())
    }
}
