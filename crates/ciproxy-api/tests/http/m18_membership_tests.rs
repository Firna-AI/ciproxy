//! Current queue membership stays separate from retained execution and history.

use axum::http::StatusCode;
use ciproxy_store_interface::billing::state::Tier;
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::github::DemandPhase;
use serde_json::Value;

use super::fixture_io::json;
use super::m18_jobs_support::{actions, demand, runner};
use super::m18_membership_support;
use super::m18_stripe::Stripe;
use super::m18_support::{Pages, at};

#[tokio::test]
async fn m18_takeouts_preserve_history_and_current_queue_banner_and_usage_scope() {
    let pages = Pages::new(Stripe::new().provider());
    let admin = pages.fixture.login(7, Some(Role::Admin)).await;
    pages.save(&pages.state(Tier::Team));
    let current = runner(&pages, None, 1001);
    demand(&pages, &current, None, DemandPhase::Assigned, None, None);
    let leaving = m18_membership_support::organization(&pages, 200, "leaving");
    let left_queue = m18_membership_support::runner(&pages, &leaving, 1002);
    demand(&pages, &left_queue, None, DemandPhase::Assigned, None, None);
    let busy = m18_membership_support::runner(&pages, &leaving, 1003);
    demand(
        &pages,
        &busy,
        None,
        DemandPhase::Started,
        Some("2026-11-06T11:59:00Z"),
        None,
    );
    let finished = actions(&pages, &busy, 31, "2026-11-06T11:58:00Z");
    let held = m18_membership_support::organization(&pages, 300, "held-leaving");
    m18_membership_support::hold(&pages, &held);
    let initial = json(pages.request(Some(&admin), "GET", "/jobs/live", None).await).await;
    assert_eq!(initial["waiting_count"], "2");
    let held_banner = json(
        pages
            .request(Some(&admin), "GET", "/banner?page=organizations", None)
            .await,
    )
    .await;
    assert_eq!(
        held_banner["banner"]["organizations"][0]["id"],
        held.id.to_string()
    );
    for organization in [&leaving, &held] {
        let response = pages
            .request(
                Some(&admin),
                "DELETE",
                &format!("/organizations/{}", organization.id),
                None,
            )
            .await;
        assert_eq!(response.status(), StatusCode::OK);
    }
    let live = json(pages.request(Some(&admin), "GET", "/jobs/live", None).await).await;
    assert_eq!(live["waiting_count"], "1");
    assert_eq!(live["waiting"].as_array().unwrap().len(), 1);
    assert_eq!(live["waiting"][0]["organization"], "customer");
    assert_eq!(live["waiting_labels"][0]["waiting_jobs"], "1");
    assert_eq!(live["visibility"], Value::Array(Vec::new()));
    assert_eq!(live["running"][0]["organization"], "leaving");
    let history = json(
        pages
            .request(Some(&admin), "GET", "/jobs/history", None)
            .await,
    )
    .await;
    assert_eq!(history["jobs"][0]["id"], finished.id.to_string());
    let banner = json(
        pages
            .request(Some(&admin), "GET", "/banner?page=organizations", None)
            .await,
    )
    .await;
    assert_eq!(banner["banner"], Value::Null);
    let usage = json(pages.request(Some(&admin), "GET", "/billing", None).await).await;
    assert_eq!(usage["organizations"], "1");
    let owner_removed = m18_membership_support::organization(&pages, 400, "owner-removed");
    pages
        .fixture
        .database
        .store
        .record_app_removal(
            pages.fixture.workspace,
            owner_removed.id,
            at("2026-11-06T12:00:00Z"),
        )
        .unwrap();
    let removed = json(pages.request(Some(&admin), "GET", "/jobs/live", None).await).await;
    assert_eq!(removed["waiting_count"], "1");
    assert_eq!(removed["visibility"][0]["removed"], true);
    assert_eq!(
        removed["visibility"][0]["github_url"],
        pages.fixture.state.config.github_install_url
    );
    assert_eq!(
        pages
            .fixture
            .database
            .store
            .jobs(pages.fixture.workspace, leaving.id)
            .unwrap()
            .len(),
        1
    );
}
