//! Real overflow observations with controlled credential and backend boundaries.

use std::sync::Arc;

use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_api::pages::PageServices;
use ciproxy_overflow::api::OverflowConfig;
use ciproxy_overflow::clock::ClockMock;
use ciproxy_overflow::crypto::KeyCipherMock;
use ciproxy_overflow::provider::{BackendFactoryMock, BoatAccountApi, Capabilities};
use ciproxy_overflow::secrets::SecretBytes;
use ciproxy_overflow::service::{OverflowDependencies, OverflowService};
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::boat::BoatAccount;
use ciproxy_store_interface::overflow::account::SizeOffer;
use ciproxy_store_interface::secrets::EncryptedBoatKey;
use rust_decimal::Decimal;
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use super::m18_support::{Pages, at};

pub(super) fn attach(pages: &mut Pages) {
    attach_with_account(pages, Arc::new(Unimock::new(())));
}

pub(super) fn attach_with_account(pages: &mut Pages, account: Arc<dyn BoatAccountApi>) {
    pages
        .fixture
        .database
        .store
        .connect_boat(
            pages.fixture.workspace,
            &BoatAccount {
                id: Uuid::new_v4(),
                account_name: "fit-test".into(),
                connected_at: at("2026-11-01T00:00:00Z"),
                retired_at: None,
                rejected_at: None,
            },
            &EncryptedBoatKey::new(vec![17]),
            1_000_000,
        )
        .unwrap();
    catalog(pages, false, false);
    let sizing = RootSizing {
        image_root_bytes: 58_300_000_000,
    };
    let capabilities = Capabilities {
        lifecycle: Arc::new(Unimock::new(())),
        capacity: Arc::new(Unimock::new(())),
        files: Arc::new(Unimock::new(())),
        account,
        launcher: Arc::new(Unimock::new(())),
        inspection: Arc::new(Unimock::new(())),
    };
    let overflow = OverflowService::new(
        OverflowConfig {
            deployment: "fit-test".into(),
            shared_snapshot: "test-image".into(),
            root_sizing: sizing,
            exit_report_url: Secret::new("https://app.ciproxy.example/overflow/end"),
        },
        OverflowDependencies {
            store: pages.fixture.database.store.clone(),
            cipher: Arc::new(Unimock::new(
                KeyCipherMock::open
                    .each_call(matching!())
                    .answers(&|_, _, _| Ok(SecretBytes::new(b"offline-fit-test".to_vec()))),
            )),
            factory: Arc::new(Unimock::new(
                BackendFactoryMock::backend
                    .each_call(matching!())
                    .answers_arc(Arc::new(move |_, _, _| Ok(capabilities.clone()))),
            )),
            runners: Arc::new(Unimock::new(())),
            tokens: Arc::new(Unimock::new(())),
            clock: Arc::new(Unimock::new(
                ClockMock::now
                    .each_call(matching!())
                    .returns(at("2026-11-06T12:00:00Z")),
            )),
        },
    );
    pages.services = Arc::new(PageServices {
        billing: pages.services.billing.clone(),
        metering: pages.services.metering.clone(),
        price_lists: pages.services.price_lists.clone(),
        paid_review: None,
        overflow: Some(Arc::new(overflow)),
        boat_root_sizing: Some(sizing),
    });
}

pub(super) fn catalog(pages: &Pages, fits: bool, refused: bool) {
    let mut session = pages
        .fixture
        .database
        .store
        .try_lock_overflow(pages.fixture.workspace)
        .unwrap()
        .unwrap();
    let mut account = session.snapshot().unwrap().accounts.remove(0);
    account.state.checked_at = Some(at("2026-11-06T12:00:00Z"));
    account.state.sizes = vec![SizeOffer {
        name: "observed-size".into(),
        vcpus: if fits { 128 } else { 2 },
        memory_bytes: if fits {
            1_024_000_000_000
        } else {
            8_000_000_000
        },
        disk_bytes: if fits {
            100_000_000_000
        } else {
            60_000_000_000
        },
        hourly_usd: Decimal::ONE,
    }];
    account.state.refused_sizes = if refused {
        vec!["observed-size".into()]
    } else {
        Vec::new()
    };
    session.save_account(&account).unwrap();
}

pub(super) fn sizing(pages: &mut Pages, known: bool) {
    pages.services = Arc::new(PageServices {
        billing: pages.services.billing.clone(),
        metering: pages.services.metering.clone(),
        price_lists: pages.services.price_lists.clone(),
        paid_review: None,
        overflow: pages.services.overflow.clone(),
        boat_root_sizing: known.then_some(RootSizing {
            image_root_bytes: 58_300_000_000,
        }),
    });
}
