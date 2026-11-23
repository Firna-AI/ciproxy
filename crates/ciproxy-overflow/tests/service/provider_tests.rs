//! Provider settings and label authority at the public service boundary.

use ciproxy_overflow::api::{Overflow, StartOutcome, WaitReason};
use ciproxy_overflow::error::Error;
use ciproxy_overflow::provider::{BillingAccount, KeyAccounts, ProviderAccountApiMock};
use ciproxy_overflow::view::RefreshStatus;
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::secret::Secret;
use ciproxy_scaler::provider::MonthlySpend;
use ciproxy_store_interface::labels::{Label, LabelPlace};
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use ciproxy_store_interface::overflow::record::StartPhase;
use rust_decimal::Decimal;
use sandbox_interface::SandboxAccountCapacityMock;
use unimock::{MockFn, Unimock, matching};

use super::dispatch_support::Provider;
use super::fixtures::{at, capabilities, facts, id, record, request};
use super::harness::{Harness, Runtime};

const BOAT: OverflowProvider = OverflowProvider::Boat;

#[tokio::test]
async fn first_connection_offers_boat_at_the_end_of_each_linux_order() {
    let accounts = KeyAccounts {
        accounts: vec![BillingAccount {
            id: "personal".to_owned(),
            name: "Customer".to_owned(),
        }],
        selected_account: "personal".to_owned(),
    };
    let backend = capabilities(Unimock::new((
        ProviderAccountApiMock::check_key
            .some_call(matching!())
            .returns(Ok(accounts.clone())),
        ProviderAccountApiMock::account
            .some_call(matching!())
            .answers(&|_| Ok(facts())),
        SandboxAccountCapacityMock::account_capacity
            .some_call(matching!())
            .returns(Ok(vec![sandbox_boat::catalogue::offer("large").unwrap()])),
    )));
    let harness = Harness::new(backend, (), Runtime::Authority);
    let workspace = id(100);
    harness.database.tenant(100);
    let store = &harness.database.store;
    let mut linux = store.labels(workspace).unwrap().remove(0);
    linux.order = vec![LabelPlace::Machine];
    store.update_label(workspace, &linux).unwrap();
    let empty = Label {
        id: id(106),
        name: "linux-empty".to_owned(),
        order: vec![],
        ..linux.clone()
    };
    let mac = Label {
        id: id(107),
        name: "mac".to_owned(),
        platform: Platform::MacosArm64,
        ..linux.clone()
    };
    store.create_label(workspace, &empty).unwrap();
    store.create_label(workspace, &mac).unwrap();

    let key = Secret::new("fixture-key");
    assert_eq!(
        harness.service.check_key(BOAT, key.clone()).await.unwrap(),
        accounts
    );
    assert_eq!(
        store.overflow_provider_settings(workspace, BOAT).unwrap(),
        None
    );
    assert_eq!(store.overflow_provider_accounts(workspace).unwrap(), []);

    harness
        .service
        .connect(
            workspace,
            BOAT,
            id(108),
            key,
            "personal".to_owned(),
            20_000_000,
        )
        .await
        .unwrap();
    let labels = store.labels(workspace).unwrap();
    for (label_id, order) in [
        (
            linux.id,
            vec![LabelPlace::Machine, LabelPlace::Provider(BOAT)],
        ),
        (empty.id, vec![LabelPlace::Provider(BOAT)]),
        (mac.id, vec![LabelPlace::Machine]),
    ] {
        assert_eq!(
            labels
                .iter()
                .find(|label| label.id == label_id)
                .unwrap()
                .order,
            order
        );
    }
    let view = harness.service.view(workspace).await.unwrap();
    let boat = view.provider(BOAT).unwrap();
    let account = boat.account.as_ref().unwrap();
    assert_eq!(account.id, id(108));
    assert_eq!(account.account_name, "Customer");
    assert_eq!(boat.limit_usd, Some(Decimal::from(20)));
    assert!(boat.offered);
}

#[tokio::test]
async fn disconnect_keeps_the_offer_and_reconnect_respects_an_edited_order() {
    let harness = Harness::new(Provider::new(false).backend, (), Runtime::Authority);
    let workspace = id(1);
    let store = &harness.database.store;
    let mut label = store.labels(workspace).unwrap().remove(0);

    harness.service.disconnect(workspace, BOAT).await.unwrap();
    let disconnected = harness.boat_view().await;
    assert_eq!(disconnected.account, None);
    assert!(disconnected.offered);
    assert_eq!(disconnected.limit_usd, Some(Decimal::from(200)));
    assert_eq!(store.labels(workspace).unwrap(), [label.clone()]);

    label.order = vec![LabelPlace::Machine];
    store.update_label(workspace, &label).unwrap();
    harness
        .service
        .connect(
            workspace,
            BOAT,
            id(8),
            Secret::new("fixture-reconnect"),
            "personal".to_owned(),
            30_000_000,
        )
        .await
        .unwrap();
    let reconnected = harness.boat_view().await;
    assert_eq!(reconnected.account.unwrap().id, id(8));
    assert!(reconnected.offered);
    assert_eq!(reconnected.limit_usd, Some(Decimal::from(30)));
    assert_eq!(store.labels(workspace).unwrap(), [label]);
}

#[tokio::test]
async fn boat_limit_counts_costs_from_the_retired_key() {
    let provider = Provider::new(false);
    let harness = Harness::new(provider.backend, (), Runtime::Start);
    let workspace = id(1);
    let mut ended = record(StartPhase::Ended);
    ended.ends_at = Some(at());
    ended.start_usd = Some(Decimal::ONE);
    ended.runtime_complete = true;
    harness.seed(ended);
    harness
        .service
        .connect(
            workspace,
            BOAT,
            id(8),
            Secret::new("fixture-replacement"),
            "personal".to_owned(),
            1_000_000,
        )
        .await
        .unwrap();

    let mut next = request();
    next.delivery_id = id(21);
    next.runner_id = id(22);
    assert_eq!(
        harness.service.start(workspace, next).await.unwrap(),
        StartOutcome::Waiting(WaitReason::SpendLimit {
            resumes_at: "2026-12-01T00:00:00Z".parse().unwrap(),
        })
    );
    assert_eq!(provider.creates.lock().unwrap().len(), 0);
    let boat = harness.boat_view().await;
    assert_eq!(boat.spend.confirmed_usd, Decimal::ONE);
    assert_eq!(
        boat.scaler_account(at()).unwrap().spend,
        MonthlySpend {
            spent: 1_000_000,
            limit: 1_000_000,
            next_month_spent: Some(0),
        }
    );

    harness
        .database
        .store
        .save_overflow_provider_spend_limit(workspace, BOAT, Some(1_462_000))
        .unwrap();
    let outcome = harness.service.resume(workspace, id(21)).await.unwrap();
    assert!(
        matches!(outcome, StartOutcome::Pending(record) if record.phase == StartPhase::Creating)
    );
    assert_eq!(provider.creates.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn unavailable_boat_retains_its_saved_view_and_allows_disconnect() {
    let harness = Harness::without_provider(Runtime::Cancellation);
    let workspace = id(1);
    let view = harness.service.view(workspace).await.unwrap();
    let boat = view.provider(BOAT).unwrap();
    assert_eq!(boat.refresh, RefreshStatus::Unavailable);
    assert_eq!(boat.account.as_ref().unwrap().id, id(2));
    assert_eq!(boat.limit_usd, Some(Decimal::from(200)));
    assert!(boat.offered);
    assert!(matches!(
        boat.scaler_account(at()),
        Err(Error::ProviderUnavailable)
    ));

    harness.service.disconnect(workspace, BOAT).await.unwrap();
    let view = harness.service.view(workspace).await.unwrap();
    let boat = view.provider(BOAT).unwrap();
    assert_eq!(boat.account, None);
    assert_eq!(boat.limit_usd, Some(Decimal::from(200)));
    assert!(boat.offered);
}
