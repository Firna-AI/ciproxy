//! Provider call counters and independent settings writes for authority tests.

use std::sync::{Arc, Mutex};

use ciproxy_overflow::maintenance::{GuestInspectionMock, LaunchState};
use ciproxy_overflow::provider::{BoatAccountApiMock, Capabilities, GuestLauncherMock};
use ciproxy_store_interface::store::Store;
use sandbox_interface::{
    BackendManagedSandbox, BackendSandbox, ProviderRef, SandboxAccountCapacityMock,
    SandboxFilesMock, SandboxLifecycleMock, SandboxState,
};
use unimock::{MockFn, Unimock, matching};

use super::fixtures::{capabilities, facts, id};
use super::label_hooks::{LabelHook, LabelStage};
use super::organization_hooks::{OrganizationHook, Stage};

#[derive(Default, Debug)]
pub(super) struct Counts {
    pub(super) creates: usize,
    pub(super) recoveries: usize,
    pub(super) inputs: usize,
    pub(super) bootstraps: usize,
    pub(super) prepares: usize,
    pub(super) launches: usize,
    pub(super) deletes: usize,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum DisableAt {
    Never,
    Prepared,
    InputWritten,
    KeyDecrypted,
}

pub(super) struct Provider {
    pub(super) backend: Capabilities,
    pub(super) counts: Arc<Mutex<Counts>>,
    pub(super) store: Arc<Mutex<Option<Arc<dyn Store>>>>,
    pub(super) launched: Arc<Mutex<Option<LaunchState>>>,
    pub(super) recovered: Arc<Mutex<bool>>,
    pub(super) inspect_state: Arc<Mutex<SandboxState>>,
    pub(super) organization: OrganizationHook,
    pub(super) label: LabelHook,
}

impl Provider {
    pub(super) fn new(disable_at: DisableAt) -> Self {
        let counts = Arc::new(Mutex::new(Counts::default()));
        let creating = counts.clone();
        let recovering_counts = counts.clone();
        let writing = counts.clone();
        let launching = counts.clone();
        let deleting = counts.clone();
        let preparing = counts.clone();
        let store = Arc::new(Mutex::new(None::<Arc<dyn Store>>));
        let prepared = store.clone();
        let written = store.clone();
        let launched = Arc::new(Mutex::new(Some(LaunchState::NotStarted)));
        let inspection = launched.clone();
        let recovered = Arc::new(Mutex::new(true));
        let recovering = recovered.clone();
        let inspect_state = Arc::new(Mutex::new(SandboxState::Ready));
        let inspecting = inspect_state.clone();
        let label = LabelHook::default();
        let account_label = label.clone();
        let capacity_label = label.clone();
        let created_label = label.clone();
        let recovered_label = label.clone();
        let inspected_label = label.clone();
        let listed_label = label.clone();
        let written_label = label.clone();
        let prepared_label = label.clone();
        let launched_label = label.clone();
        let launch_inspected_label = label.clone();
        let usage_label = label.clone();
        let destroyed_label = label.clone();
        let organization = OrganizationHook::default();
        let created_org = organization.clone();
        let prepared_org = organization.clone();
        let written_org = organization.clone();
        let backend = capabilities(
            Unimock::new((
                GuestInspectionMock::inspect_launch
                    .some_call(matching!(_, _))
                    .answers_arc(Arc::new(move |_, _, _| {
                        launch_inspected_label.apply(LabelStage::LaunchInspection);
                        inspection
                            .lock()
                            .unwrap()
                            .ok_or(ciproxy_overflow::error::Error::ProviderUnavailable)
                    })),
                BoatAccountApiMock::account
                    .some_call(matching!())
                    .answers_arc(Arc::new(move |_| {
                        account_label.apply(LabelStage::Account);
                        Ok(facts())
                    })),
                SandboxAccountCapacityMock::account_capacity
                    .some_call(matching!())
                    .answers_arc(Arc::new(move |_| {
                        capacity_label.apply(LabelStage::Capacity);
                        Ok(vec![sandbox_boat::catalogue::offer("large").unwrap()])
                    })),
                SandboxLifecycleMock::create_sandbox
                    .some_call(matching!(_))
                    .answers_arc(Arc::new(move |_, request| {
                        creating.lock().unwrap().creates += 1;
                        created_org.apply(Stage::Created);
                        created_label.apply(LabelStage::Create);
                        Ok(BackendSandbox {
                            provider_ref: ProviderRef::new(
                                if request.sandbox_id.as_uuid() == id(12) {
                                    "bx_23456789"
                                } else {
                                    "bx_3456789a"
                                },
                            ),
                            state: SandboxState::Ready,
                        })
                    })),
                SandboxLifecycleMock::recover_sandbox_create
                    .some_call(matching!(_))
                    .answers_arc(Arc::new(move |_, _| {
                        recovering_counts.lock().unwrap().recoveries += 1;
                        recovered_label.apply(LabelStage::Recover);
                        Ok(recovering.lock().unwrap().then(sandbox))
                    })),
                SandboxLifecycleMock::inspect_sandbox
                    .some_call(matching!(_))
                    .answers_arc(Arc::new(move |_, provider_ref| {
                        inspected_label.apply(LabelStage::Inspect);
                        Ok(BackendSandbox {
                            provider_ref,
                            state: *inspecting.lock().unwrap(),
                        })
                    })),
                SandboxLifecycleMock::list_managed_sandboxes
                    .some_call(matching!(_))
                    .answers_arc(Arc::new(move |_, _| {
                        listed_label.apply(LabelStage::Inventory);
                        Ok(vec![BackendManagedSandbox {
                            provider_ref: ProviderRef::new("bx_23456789"),
                            state: SandboxState::Ready,
                            sandbox_id: None,
                            operation_id: None,
                            consumer: None,
                            lifetime: None,
                        }])
                    })),
                SandboxFilesMock::write_file
                    .some_call(matching!(_))
                    .answers_arc(Arc::new(move |_, request| {
                        if !request.bytes.is_empty() {
                            writing.lock().unwrap().inputs += 1;
                            written_org.apply(Stage::InputWritten);
                            written_label.apply(LabelStage::Input);
                            if disable_at == DisableAt::InputWritten {
                                disable(written.lock().unwrap().as_ref().unwrap());
                            }
                        } else {
                            writing.lock().unwrap().bootstraps += 1;
                            written_label.apply(LabelStage::Bootstrap);
                        }
                        Ok(())
                    })),
                GuestLauncherMock::prepare
                    .some_call(matching!(_, _))
                    .answers_arc(Arc::new(move |_, _, _| {
                        preparing.lock().unwrap().prepares += 1;
                        prepared_org.apply(Stage::Prepared);
                        prepared_label.apply(LabelStage::Prepare);
                        if disable_at == DisableAt::Prepared {
                            disable(prepared.lock().unwrap().as_ref().unwrap());
                        }
                        Ok(())
                    })),
                GuestLauncherMock::launch
                    .some_call(matching!(_, _))
                    .answers_arc(Arc::new(move |_, _, _| {
                        launching.lock().unwrap().launches += 1;
                        launched_label.apply(LabelStage::Launch);
                        Ok(())
                    })),
                BoatAccountApiMock::usage
                    .some_call(matching!(_, _, _))
                    .answers_arc(Arc::new(move |_, _, _, _| {
                        usage_label.apply(LabelStage::Usage);
                        Ok(None)
                    })),
                SandboxLifecycleMock::destroy_sandbox
                    .some_call(matching!(_))
                    .answers_arc(Arc::new(move |_, _| {
                        deleting.lock().unwrap().deletes += 1;
                        destroyed_label.apply(LabelStage::Destroy);
                        Ok(())
                    })),
            ))
            .no_verify_in_drop(),
        );
        Self {
            backend,
            counts,
            store,
            launched,
            recovered,
            inspect_state,
            organization,
            label,
        }
    }
}

pub(super) fn disable(store: &Arc<dyn Store>) {
    let mut settings = store.settings(id(1)).unwrap();
    settings.overflow_enabled = false;
    store.save_settings(id(1), &settings).unwrap();
}

fn sandbox() -> BackendSandbox {
    BackendSandbox {
        provider_ref: ProviderRef::new("bx_23456789"),
        state: SandboxState::Ready,
    }
}
