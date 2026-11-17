//! Report failures distinguish unmeasured startup state from measured zero.

use reqwest::StatusCode;

use crate::support::{Fixture, facts};

#[tokio::test]
async fn reports_wait_for_the_first_budget_and_accept_a_measured_zero_budget() {
    let mut fixture = Fixture::new().await;
    fixture.restart_waiting_for_budget().await;
    assert_eq!(
        fixture.report_response().await.status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    fixture.budget(0).await;
    let report = fixture.report().await;
    assert_eq!((report.used_bytes, report.capacity_bytes), (0, 0));
}

#[tokio::test]
async fn a_later_budget_does_not_hide_failed_disk_recovery() {
    let mut fixture = Fixture::new().await;
    let cache = fixture.cache_path();
    let retained = fixture.root.path().join("retained-cache");
    std::fs::rename(&cache, &retained).unwrap();
    std::fs::write(&cache, b"not a cache directory").unwrap();
    fixture.restart_waiting_for_budget().await;
    std::fs::remove_file(&cache).unwrap();
    std::fs::rename(retained, cache).unwrap();
    fixture.budget(0).await;
    assert_eq!(
        fixture.report_response().await.status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
}

#[tokio::test]
async fn unread_saved_counts_remain_unavailable_after_cache_events() {
    let fixture = Fixture::new().await;
    let saved = fixture.saved_counts_path();
    std::fs::write(&saved, b"unread saved counts").unwrap();
    assert_eq!(
        fixture.report_response().await.status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    let job = fixture
        .job("unknown-counts", facts("refs/heads/main"))
        .await;
    job.save("counted", "v1", "contents").await;
    assert_eq!(job.contents("counted", "v1", &[]).await, "contents");
    let fallback = job
        .client
        .post(format!("{}/ciproxy/fallback", fixture.machine))
        .header("x-ciproxy-session", job.token.expose_secret())
        .send()
        .await
        .unwrap();
    assert_eq!(fallback.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        fixture.report_response().await.status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(std::fs::read(saved).unwrap(), b"unread saved counts");
}
