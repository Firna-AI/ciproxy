//! Complete physical label placement and thirty-day identity-based job summaries.

use std::sync::Arc;

use axum::http::StatusCode;
use ciproxy_overflow::error::Error;
use ciproxy_overflow::provider::ProviderAccountApiMock;
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::labels::WhenNothingFits;
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use serde_json::{Value, json};
use unimock::{MockFn, Unimock, matching};

use super::capacity_fixture;
use super::fixture::{Browser, Fixture};
use super::fixture_io::json;
use super::m18_fit_support::{attach_with_account, catalog};
use super::m18_jobs_support::{actions, demand, runner};
use super::m18_stripe::Stripe;
use super::m18_support::{Pages, at};
use super::m20_label_support::retired_name;
use super::machine_tests::register;

#[tokio::test]
async fn m20_labels_supply_current_placement_allocations_and_retained_thirty_day_counts() {
    let mut pages = Pages::new(Stripe::new().provider());
    pages.fixture.state.capacity = capacity_fixture::standard();
    let admin = pages.fixture.login(7, Some(Role::Admin)).await;
    let (machine, _connection) = register(&pages.fixture, &admin).await;
    let unobserved = labels(&pages.fixture, &admin).await;
    assert_eq!(
        unobserved[0]["fit"]["providers"][0]["connected"],
        Value::Null
    );
    assert_eq!(unobserved[0]["fit"]["fitting_places"], Value::Null);
    attach_with_account(
        &mut pages,
        Arc::new(Unimock::new(
            ProviderAccountApiMock::account
                .some_call(matching!())
                .answers(&|_| Err(Error::ProviderUnavailable)),
        )),
    );
    pages.fixture.state.overflow = pages.services.overflow.clone();
    catalog(&pages, true, true);
    let own = runner(&pages, Some(machine), 1001);
    actions(&pages, &own, 501, "2026-10-20T10:00:00Z");
    demand(
        &pages,
        &own,
        Some(501),
        DemandPhase::Completed,
        Some("2026-10-20T10:00:00Z"),
        Some("2026-10-20T10:01:00Z"),
    );
    let separate = runner(&pages, Some(machine), 1002);
    let mut untimed = demand(&pages, &separate, None, DemandPhase::Started, None, None);
    let old = runner(&pages, Some(machine), 1003);
    actions(&pages, &old, 502, "2026-09-01T10:00:00Z");
    let body = labels(&pages.fixture, &admin).await;
    assert_eq!(body[0]["jobs_last_30_days"], Value::Null);
    untimed.started_at = Some(at("2026-11-05T10:00:00Z"));
    pages
        .fixture
        .database
        .store
        .save_github_job(
            pages.fixture.workspace,
            &untimed,
            at("2026-11-06T12:00:00Z"),
        )
        .unwrap();
    let body = labels(&pages.fixture, &admin).await;
    let fit = &body[0]["fit"];
    assert_eq!(body[0]["jobs_last_30_days"], "2");
    assert_eq!(body[0]["job_count_since"], "2026-10-07T12:00:00Z");
    assert_eq!(body[0]["job_count_until"], "2026-11-06T12:00:00Z");
    assert_eq!(
        fit["fitting_places"],
        json!([{"kind":"machine"},{"kind":"overflow","provider":"boat"}])
    );
    assert_eq!(fit["providers"][0]["connected"], true);
    assert_eq!(fit["providers"][0]["fit"]["state"], "fitting");
    assert_eq!(fit["providers"][0]["fit"]["refused"], true);
    assert_eq!(
        fit["providers"][0]["required_root_disk_bytes"],
        "78935855104"
    );
    assert_eq!(
        fit["providers"][0]["largest_size"]["size"]["disk_bytes"],
        "100000000000"
    );
    assert_eq!(fit["machines"][0]["vcpus"], "12");
    assert_eq!(fit["machines"][0]["memory_bytes"], "48000000000");
    assert_eq!(fit["machines"][0]["job_disk_bytes"], "180000000000");
    assert_eq!(
        fit["machines"][0]["required_root_disk_bytes"],
        "80000000000"
    );
    assert_eq!(fit["machines"][0]["concurrent_jobs"], "2");
    let mut label = pages
        .fixture
        .database
        .store
        .labels(pages.fixture.workspace)
        .unwrap()
        .remove(0);
    label.size.vcpus = 128;
    label.size.memory_gb = 1024;
    pages
        .fixture
        .database
        .store
        .update_label(pages.fixture.workspace, &label)
        .unwrap();
    let body = labels(&pages.fixture, &admin).await;
    assert_eq!(
        body[0]["fit"]["fitting_places"],
        json!([{"kind":"overflow","provider":"boat"}])
    );
    catalog(&pages, false, false);
    let body = labels(&pages.fixture, &admin).await;
    assert_eq!(body[0]["fit"]["fitting_places"], json!([]));
    assert_eq!(
        body[0]["fit"]["providers"][0]["fit"]["state"],
        "no_size_fits"
    );
    assert_eq!(
        body[0]["fit"]["providers"][0]["largest_size"]["size"]["disk_bytes"],
        "60000000000"
    );
    label.when_nothing_fits = WhenNothingFits::LargestOverflow;
    pages
        .fixture
        .database
        .store
        .update_label(pages.fixture.workspace, &label)
        .unwrap();
    let body = labels(&pages.fixture, &admin).await;
    assert_eq!(
        body[0]["fit"]["providers"][0]["fit"]["state"],
        "largest_fallback"
    );
    assert_eq!(
        body[0]["fit"]["fitting_places"],
        json!([{"kind":"overflow","provider":"boat"}])
    );
    pages.fixture.state.capacity = capacity_fixture::unavailable();
    let body = labels(&pages.fixture, &admin).await;
    assert_eq!(
        body[0]["fit"]["providers"][0]["fit"]["state"],
        "unavailable"
    );
    assert_eq!(body[0]["fit"]["fitting_places"], Value::Null);
    pages.fixture.state.capacity = capacity_fixture::standard();
    let draft = json!({"name":"draft", "vcpus":128, "memory_gb":1024, "disk_gb":20, "platform":"linux_x64", "order":[{"kind":"machine"},{"kind":"overflow","provider":"boat"}], "when_nothing_fits":"largest_overflow"});
    let preview = json(
        pages
            .fixture
            .request(
                Some(&admin),
                "POST",
                &pages.fixture.path("/labels/preview"),
                Some(draft),
            )
            .await,
    )
    .await;
    assert_eq!(preview["providers"][0]["fit"]["state"], "largest_fallback");
    assert_eq!(
        preview["providers"][0]["required_root_disk_bytes"],
        "78935855104"
    );
    retired_name(&pages, &own, &label);
    let body = labels(&pages.fixture, &admin).await;
    let reused = body
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["label"]["name"] == "reused")
        .unwrap();
    assert_eq!(reused["jobs_last_30_days"], "0");
    let current_account = {
        let mut session = pages
            .fixture
            .database
            .store
            .try_lock_overflow(pages.fixture.workspace)
            .unwrap()
            .unwrap();
        let mut account = session.snapshot().unwrap().accounts.remove(0);
        account.state.checked_at = None;
        session.save_account(&account).unwrap();
        account.account.id
    };
    let failed_read = labels(&pages.fixture, &admin).await;
    assert_eq!(
        failed_read[0]["fit"]["providers"][0]["connected"],
        Value::Null
    );
    assert_eq!(failed_read[0]["fit"]["fitting_places"], Value::Null);
    pages
        .fixture
        .database
        .store
        .reject_overflow_key(
            pages.fixture.workspace,
            current_account,
            at("2026-11-06T12:00:00Z"),
        )
        .unwrap();
    let missing_catalog = labels(&pages.fixture, &admin).await;
    assert_eq!(missing_catalog[0]["fit"]["providers"][0]["connected"], true);
    assert_eq!(
        missing_catalog[0]["fit"]["providers"][0]["fit"]["state"],
        "unavailable"
    );
    assert_eq!(missing_catalog[0]["fit"]["fitting_places"], Value::Null);
    pages
        .fixture
        .database
        .store
        .disconnect_overflow_provider(
            pages.fixture.workspace,
            OverflowProvider::Boat,
            at("2026-11-06T12:00:00Z"),
        )
        .unwrap();
    label.size.vcpus = 2;
    label.size.memory_gb = 8;
    pages
        .fixture
        .database
        .store
        .update_label(pages.fixture.workspace, &label)
        .unwrap();
    let disconnected = labels(&pages.fixture, &admin).await;
    assert_eq!(
        disconnected[0]["fit"]["fitting_places"],
        json!([{"kind":"machine"}])
    );
    assert_eq!(disconnected[0]["fit"]["providers"][0]["connected"], false);
    assert_eq!(
        disconnected[0]["fit"]["providers"][0]["fit"]["state"],
        "unavailable"
    );
    label.size.vcpus = 128;
    label.size.memory_gb = 1024;
    pages
        .fixture
        .database
        .store
        .update_label(pages.fixture.workspace, &label)
        .unwrap();
    let disconnected = labels(&pages.fixture, &admin).await;
    assert_eq!(disconnected[0]["fit"]["fitting_places"], json!([]));
    assert_eq!(disconnected[0]["fit"]["providers"][0]["connected"], false);
    assert_eq!(
        disconnected[0]["fit"]["providers"][0]["fit"]["state"],
        "unavailable"
    );
    pages.fixture.state.capacity = capacity_fixture::unavailable();
    let unknown = labels(&pages.fixture, &admin).await;
    assert_eq!(unknown[0]["fit"]["providers"][0]["connected"], false);
    assert_eq!(unknown[0]["fit"]["fitting_places"], Value::Null);
    pages.fixture.state.capacity = capacity_fixture::standard();
    pages.fixture.state.overflow = None;
    let unobserved = labels(&pages.fixture, &admin).await;
    assert_eq!(
        unobserved[0]["fit"]["providers"][0]["connected"],
        Value::Null
    );
    assert_eq!(unobserved[0]["fit"]["fitting_places"], Value::Null);
    let viewer = pages.fixture.login(8, Some(Role::Viewer)).await;
    assert_eq!(pages.fixture.request(Some(&viewer), "POST", &pages.fixture.path("/labels/preview"), Some(json!({"name":"draft", "vcpus":2,"memory_gb":8,"disk_gb":20,"platform":"linux_x64", "order":[{"kind":"machine"},{"kind":"overflow","provider":"boat"}],"when_nothing_fits":"wait"}))).await.status(), StatusCode::FORBIDDEN);
}

async fn labels(fixture: &Fixture, browser: &Browser) -> Value {
    let response = fixture
        .request(Some(browser), "GET", &fixture.path("/labels"), None)
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    json(response).await
}
