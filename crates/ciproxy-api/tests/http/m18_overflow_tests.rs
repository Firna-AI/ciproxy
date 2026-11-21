//! Configured live observations use the real overflow service and PgStore.

use std::sync::Arc;

use axum::http::StatusCode;
use ciproxy_api::pages::PageServices;
use ciproxy_overflow::api::OverflowConfig;
use ciproxy_overflow::clock::ClockMock;
use ciproxy_overflow::service::{OverflowDependencies, OverflowService};
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::boat::BoatAccount;
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::secrets::EncryptedBoatKey;
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use super::fixture_io::json;
use super::m18_stripe::Stripe;
use super::m18_support::{Pages, at};

#[tokio::test]
async fn m18_live_exposes_saved_provider_refusals_without_keys_or_starting_sandboxes() {
    let mut pages = Pages::new(Stripe::new().provider());
    let admin = pages.fixture.login(7, Some(Role::Admin)).await;
    let account = BoatAccount {
        id: Uuid::new_v4(),
        account_name: "test-account".into(),
        connected_at: at("2026-11-01T12:00:00Z"),
        retired_at: None,
        rejected_at: Some(at("2026-11-05T12:00:00Z")),
    };
    pages
        .fixture
        .database
        .store
        .connect_boat(
            pages.fixture.workspace,
            &account,
            &EncryptedBoatKey::new(vec![17, 18, 19]),
            1_000_000,
        )
        .unwrap();
    {
        let mut session = pages
            .fixture
            .database
            .store
            .try_lock_overflow(pages.fixture.workspace)
            .unwrap()
            .unwrap();
        let mut saved = session.snapshot().unwrap().accounts.remove(0);
        saved.account.rejected_at = account.rejected_at;
        saved.state.checked_at = Some(at("2026-11-06T12:00:00Z"));
        saved.state.billing_required = true;
        saved.state.retry_at = Some(at("2026-11-06T13:00:00Z"));
        saved.state.refused_sizes = vec!["unavailable-size".into()];
        session.save_account(&saved).unwrap();
    }
    let overflow = OverflowService::new(
        OverflowConfig {
            deployment: "http-test".into(),
            shared_snapshot: "test-image".into(),
            root_sizing: Default::default(),
            exit_report_url: Secret::new("https://app.ciproxy.example/overflow/end"),
        },
        OverflowDependencies {
            store: pages.fixture.database.store.clone(),
            cipher: Arc::new(Unimock::new(())),
            factory: Arc::new(Unimock::new(())),
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
        boat_root_sizing: None,
        billing: pages.services.billing.clone(),
        metering: pages.services.metering.clone(),
        price_lists: pages.services.price_lists.clone(),
        paid_review: None,
        overflow: Some(Arc::new(overflow)),
    });
    let response = pages.request(Some(&admin), "GET", "/jobs/live", None).await;
    assert_eq!(response.status(), StatusCode::OK);
    let live = json(response).await;
    assert_eq!(live["overflow"]["key_rejected"], true);
    assert_eq!(live["overflow"]["billing_required"], true);
    assert_eq!(live["overflow"]["retry_at"], "2026-11-06T13:00:00Z");
    assert_eq!(live["overflow"]["limit_cents"], "100");
    assert_eq!(live["overflow"]["counted_cents"], "0");
    assert_eq!(live["overflow"]["limit_reached"], false);
    assert_eq!(live["overflow"]["next_month"], "2026-12-01T00:00:00Z");
    assert_eq!(live["overflow"]["refused_sizes"][0], "unavailable-size");
    assert_eq!(live["overflow"].get("key"), None);
    assert_eq!(
        pages
            .fixture
            .database
            .store
            .try_lock_overflow(pages.fixture.workspace)
            .unwrap()
            .unwrap()
            .snapshot()
            .unwrap()
            .starts
            .len(),
        0
    );
}
