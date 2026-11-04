//! Accepted range replacements must not publish earlier bytes after capture fails.

use serde_json::json;

use crate::support::uploads::chunked;
use crate::support::{Fixture, facts};

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
