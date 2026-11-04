//! Protect local clear, disk recovery, eviction, and report windows.

use ciproxy_protocol::cache::JobCacheUsage;
use serde_json::json;

use crate::support::{Fixture, facts};

#[tokio::test]
async fn concurrent_vms_share_one_budget_and_resize_releases_staged_bytes() {
    let fixture = Fixture::with_disk(false, 12).await;
    let first = fixture
        .job("concurrent-one", facts("refs/heads/main"))
        .await;
    let second = fixture
        .job("concurrent-two", facts("refs/heads/main"))
        .await;
    tokio::join!(
        first.save("first", "v1", "111111"),
        second.save("second", "v1", "222222")
    );
    assert_eq!(
        (
            fixture.report().await.used_bytes,
            fixture.report().await.capacity_bytes
        ),
        (12, 12)
    );
    let pending = first.reserve("pending-budget", "v1").await;
    assert_eq!(
        first
            .client
            .put(pending)
            .body("12345678")
            .send()
            .await
            .unwrap()
            .status(),
        201
    );
    assert_eq!(fixture.report().await.used_bytes, 8);
    fixture.budget(4).await;
    assert_eq!(
        (
            fixture.report().await.used_bytes,
            fixture.report().await.capacity_bytes
        ),
        (0, 4)
    );
    first.finalize("pending-budget", "v1", 8).await;
    assert_eq!(fixture.report().await.used_bytes, 0);
    assert_eq!(
        fixture.upstream().await["entries"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
}

#[tokio::test]
async fn corruption_falls_back_and_clear_removes_orphaned_entry_files() {
    let fixture = Fixture::new().await;
    let job = fixture.job("corruption", facts("refs/heads/main")).await;
    job.save("corrupt", "v1", "original").await;
    let archive = std::fs::read_dir(fixture.root.path().join("cache"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path.extension().is_some_and(|value| value == "blob"))
        .unwrap();
    std::fs::write(archive, b"bad-data").unwrap();
    assert_eq!(job.contents("corrupt", "v1", &[]).await, "original");
    let missing = std::fs::read_dir(fixture.root.path().join("cache"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path.extension().is_some_and(|value| value == "blob"))
        .unwrap();
    std::fs::remove_file(missing).unwrap();
    let restored = job.lookup("corrupt", "v1", &[]).await;
    let content = job
        .client
        .get(restored["signedDownloadUrl"].as_str().unwrap())
        .send()
        .await
        .unwrap()
        .text()
        .await;
    assert!(
        matches!(content.as_deref(), Ok("original")),
        "missing-archive fallback: {content:?}"
    );
    let cache = fixture.root.path().join("cache");
    std::fs::write(cache.join("orphan.json"), b"broken metadata").unwrap();
    std::fs::write(cache.join("orphan.blob"), b"orphan archive").unwrap();
    std::fs::write(cache.join(".tmp-stale"), b"stale upload").unwrap();
    fixture.clear().await;
    assert_eq!(fixture.report().await.used_bytes, 0);
    let names: Vec<_> = std::fs::read_dir(cache)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(names, vec![std::ffi::OsString::from("reports")]);
    assert_eq!(
        fixture.upstream().await["entries"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn clear_invalidates_pending_saves_without_touching_github_or_other_disks() {
    let fixture = Fixture::new().await;
    let job = fixture.job("clear", facts("refs/heads/main")).await;
    job.save("kept-upstream", "v1", "keep").await;
    let pending = job.reserve("pending", "v1").await;
    assert_eq!(
        job.client
            .put(pending)
            .body("late")
            .send()
            .await
            .unwrap()
            .status(),
        201
    );
    std::fs::create_dir_all(fixture.root.path().join("docker")).unwrap();
    std::fs::write(fixture.root.path().join("docker/data"), b"docker").unwrap();
    fixture.clear().await;
    job.finalize("pending", "v1", 4).await;
    assert_eq!(fixture.report().await.used_bytes, 0);
    assert_eq!(
        fixture.upstream().await["entries"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        std::fs::read(fixture.root.path().join("docker/data")).unwrap(),
        b"docker"
    );
    fixture.options(json!({"lookup":false})).await;
    assert_eq!(job.lookup("kept-upstream", "v1", &[]).await["ok"], false);
    assert_eq!(job.lookup("pending", "v1", &[]).await["ok"], false);
}

#[tokio::test]
async fn eviction_and_restart_preserve_entries_and_seven_day_measured_hits() {
    let mut fixture = Fixture::new().await;
    let job = fixture.job("eviction", facts("refs/heads/main")).await;
    job.save("old", "v1", "old!").await;
    job.save("new", "v1", "new!").await;
    assert_eq!(job.contents("old", "v1", &[]).await, "old!");
    fixture.budget(4).await;
    fixture.options(json!({"lookup":false})).await;
    assert_eq!(job.lookup("new", "v1", &[]).await["ok"], false);
    assert_eq!(fixture.report().await.capacity_bytes, 4);
    assert_eq!(fixture.report().await.used_bytes, 4);
    fixture.restart().await;
    let next = fixture.job("after-restart", facts("refs/heads/main")).await;
    assert_eq!(next.contents("old", "v1", &[]).await, "old!");
    let report = fixture.report().await;
    assert_eq!(
        (report.restores_last_7_days, report.hits_last_7_days),
        (2, 2)
    );
    assert_eq!(
        report
            .jobs
            .iter()
            .find(|job| job.job_id.0 == "eviction")
            .unwrap()
            .usage,
        JobCacheUsage::Github
    );
    fixture.clock.advance(7 * 24 * 60 * 60 + 1);
    let report = fixture.report().await;
    assert_eq!(
        (report.restores_last_7_days, report.hits_last_7_days),
        (0, 0)
    );
}
