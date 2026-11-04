//! Accepted range replacements must not publish earlier bytes after capture fails.

use serde_json::json;

use crate::support::job::Job;
use crate::support::uploads::chunked;
use crate::support::{Fixture, facts};

async fn block(job: &Job, upload: &str, id: &str, data: &str) {
    let mut url = url::Url::parse(upload).unwrap();
    url.query_pairs_mut()
        .append_pair("comp", "block")
        .append_pair("blockid", id);
    assert_eq!(
        job.client
            .put(url)
            .body(data.to_owned())
            .send()
            .await
            .unwrap()
            .status(),
        201
    );
}

async fn commit(job: &Job, upload: &str, selection: &str) {
    let mut url = url::Url::parse(upload).unwrap();
    url.query_pairs_mut().append_pair("comp", "blocklist");
    assert_eq!(
        job.client
            .put(url)
            .body(format!("<BlockList>{selection}</BlockList>"))
            .send()
            .await
            .unwrap()
            .status(),
        201
    );
}

#[tokio::test]
async fn whole_upload_replaces_committed_blocks_within_the_same_budget() {
    let fixture = Fixture::with_disk(false, 3).await;
    let job = fixture
        .job("whole-replacement", facts("refs/heads/main"))
        .await;
    let upload = job.reserve("whole", "v1").await;
    block(&job, &upload, "YQ==", "abc").await;
    commit(&job, &upload, "<Latest>YQ==</Latest>").await;
    assert_eq!(fixture.report().await.used_bytes, 3);
    assert_eq!(
        job.client
            .put(upload)
            .body("XYZ")
            .send()
            .await
            .unwrap()
            .status(),
        201
    );
    job.finalize("whole", "v1", 3).await;
    assert_eq!(
        fixture.upstream().await["entries"][0]["sha256"],
        "ade099751d2ea9f3393f0f32d20c6b980dd5d3b0989dea599b966ae0d3cd5a1e"
    );
    fixture.delete_upstream().await;
    assert_eq!(job.contents("whole", "v1", &[]).await, "XYZ");
}

#[tokio::test]
async fn staged_block_replacement_preserves_the_committed_archive_until_commit() {
    let fixture = Fixture::with_disk(false, 6).await;
    let job = fixture
        .job("staged-separation", facts("refs/heads/main"))
        .await;
    let upload = job.reserve("committed", "v1").await;
    block(&job, &upload, "YQ==", "abc").await;
    commit(&job, &upload, "<Uncommitted>YQ==</Uncommitted>").await;
    block(&job, &upload, "YQ==", "XYZ").await;
    job.finalize("committed", "v1", 3).await;
    assert_eq!(
        fixture.upstream().await["entries"][0]["sha256"],
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    fixture.delete_upstream().await;
    assert_eq!(job.contents("committed", "v1", &[]).await, "abc");
}

#[tokio::test]
async fn missing_staged_capture_never_selects_the_older_committed_version() {
    let fixture = Fixture::with_disk(false, 3).await;
    let job = fixture
        .job("missing-staged", facts("refs/heads/main"))
        .await;
    let upload = job.reserve("missing", "v1").await;
    block(&job, &upload, "YQ==", "abc").await;
    commit(&job, &upload, "<Latest>YQ==</Latest>").await;
    block(&job, &upload, "YQ==", "XYZ").await;
    commit(&job, &upload, "<Latest>YQ==</Latest>").await;
    job.finalize("missing", "v1", 3).await;
    assert_eq!(
        fixture.upstream().await["entries"][0]["sha256"],
        "ade099751d2ea9f3393f0f32d20c6b980dd5d3b0989dea599b966ae0d3cd5a1e"
    );
    fixture.delete_upstream().await;
    assert_eq!(job.lookup("missing", "v1", &[]).await["ok"], false);
}

#[tokio::test]
async fn committed_block_selection_releases_unselected_staged_captures() {
    let fixture = Fixture::with_disk(false, 6).await;
    let job = fixture
        .job("discard-unselected", facts("refs/heads/main"))
        .await;
    let upload = job.reserve("selected", "v1").await;
    block(&job, &upload, "YQ==", "abc").await;
    block(&job, &upload, "Yg==", "XYZ").await;
    commit(&job, &upload, "<Latest>YQ==</Latest>").await;
    assert_eq!(fixture.report().await.used_bytes, 3);
    block(&job, &upload, "YQ==", "XYZ").await;
    commit(&job, &upload, "<Committed>YQ==</Committed>").await;
    assert_eq!(fixture.report().await.used_bytes, 3);
    job.finalize("selected", "v1", 3).await;
    fixture.delete_upstream().await;
    assert_eq!(job.contents("selected", "v1", &[]).await, "abc");
}

#[tokio::test]
async fn exhausted_range_replacement_preserves_accepted_bytes_instead_of_stale_capture() {
    let fixture = Fixture::with_disk(false, 3).await;
    let job = fixture
        .job("range-replacement", facts("refs/heads/main"))
        .await;
    let reservation: serde_json::Value = job
        .client
        .post(format!("{}/_apis/artifactcache/caches", job.origin))
        .json(&json!({"key":"replace","version":"v1","cacheSize":3}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let id = reservation["cacheId"].as_u64().unwrap();
    let upload = format!("{}/_apis/artifactcache/caches/{id}", job.origin);
    for data in ["abc", "XYZ"] {
        assert_eq!(
            job.client
                .patch(&upload)
                .header("content-range", "bytes 0-2/*")
                .body(chunked(data))
                .send()
                .await
                .unwrap()
                .status(),
            204
        );
        assert_eq!(fixture.report().await.used_bytes, 3);
    }
    assert_eq!(
        job.client
            .post(&upload)
            .json(&json!({"size":3}))
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    let state = fixture.upstream().await;
    assert_eq!(
        state["entries"][0]["sha256"],
        "ade099751d2ea9f3393f0f32d20c6b980dd5d3b0989dea599b966ae0d3cd5a1e"
    );
    fixture.delete_upstream().await;
    assert_eq!(job.contents("replace", "v1", &[]).await, "XYZ");
}
