//! Protect the runner's two cache protocols and observed transfer counts.

use ciproxy_protocol::cache::{CacheRestoreCounts, JobCacheUsage};
use serde_json::json;

use crate::support::{Fixture, facts};

#[tokio::test]
async fn v2_commits_ordered_blocks_and_counts_completed_restores_once() {
    let mut fixture = Fixture::with_disk(false, 6).await;
    fixture.options(json!({"snake":true})).await;
    let job = fixture.job("wire-v2", facts("refs/heads/main")).await;
    let upload = job.reserve("cache-blocks", "v1").await;
    for (id, contents) in [("Yg==", "DEF"), ("YQ==", "bad"), ("YQ==", "abc")] {
        let mut url = url::Url::parse(&upload).unwrap();
        url.query_pairs_mut()
            .append_pair("comp", "block")
            .append_pair("blockid", id);
        assert_eq!(
            job.client
                .put(url)
                .body(contents)
                .send()
                .await
                .unwrap()
                .status(),
            201
        );
    }
    let mut url = url::Url::parse(&upload).unwrap();
    url.query_pairs_mut().append_pair("comp", "blocklist");
    assert_eq!(job.client.put(url).body("<?xml version=\"1.0\"?><BlockList><Latest>YQ==</Latest><Latest>Yg==</Latest></BlockList>").send().await.unwrap().status(), 201);
    assert_eq!(
        job.rpc(
            "FinalizeCacheEntryUpload",
            json!({"key":"cache-blocks","version":"v1","size_bytes":"6"})
        )
        .await,
        json!({"ok":true,"entry_id":"100"})
    );
    assert_eq!(fixture.report().await.used_bytes, 6);
    fixture.budget(12).await;
    let lookup = job
        .rpc(
            "GetCacheEntryDownloadURL",
            json!({"key":"absent","version":"v1","restore_keys":["cache-blocks"]}),
        )
        .await;
    assert_eq!(lookup["matchedKey"], "cache-blocks");
    let download = lookup["signedDownloadUrl"].as_str().unwrap();
    assert_eq!(
        job.client
            .get(download)
            .header("range", "bytes=-0")
            .send()
            .await
            .unwrap()
            .status(),
        416
    );
    assert_eq!(fixture.report().await.restores_last_7_days, 0);
    assert_eq!(
        job.client
            .get(download)
            .header("range", "bytes=2-4")
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap(),
        "cDE"
    );
    let head = job.client.head(download).send().await.unwrap();
    assert_eq!(head.headers()["content-length"], "6");
    assert_eq!(fixture.report().await.restores_last_7_days, 0);
    let range = job
        .client
        .get(download)
        .header("range", "bytes=3-5")
        .send()
        .await
        .unwrap();
    assert_eq!(range.status(), 206);
    assert_eq!(range.headers()["content-range"], "bytes 3-5/6");
    assert_eq!(range.text().await.unwrap(), "DEF");
    assert_eq!(fixture.report().await.restores_last_7_days, 0);
    assert_eq!(
        job.client
            .get(download)
            .header("range", "bytes=0-2")
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap(),
        "abc"
    );
    assert_eq!(
        job.client
            .get(download)
            .header("range", "bytes=0-2")
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap(),
        "abc"
    );
    fixture.seed("remote", "v1", "cmVtb3Rl").await;
    assert_eq!(job.contents("remote", "v1", &[]).await, "remote");
    let report = fixture.report().await;
    assert_eq!(
        (
            report.used_bytes,
            report.restores_last_7_days,
            report.hits_last_7_days
        ),
        (12, 2, 1)
    );
    assert_eq!(
        report.jobs[0].usage,
        JobCacheUsage::MachineHandled(CacheRestoreCounts {
            local_entries: 1,
            github_entries: 1,
            restored_bytes: 12
        })
    );
    let upstream = fixture.upstream().await;
    assert_eq!(upstream["entries"][0]["size"], 6);
    assert_eq!(
        upstream["entries"][0]["sha256"],
        "ee9579695089f0f3dc99c3e48b6b4912a6f38cb9f98c532201b44077d659737d"
    );
    fixture.delete_upstream().await;
    let later = fixture
        .job("after-github-delete", facts("refs/heads/main"))
        .await;
    assert_eq!(later.contents("remote", "v1", &[]).await, "remote");
    fixture.restart().await;
    let restarted = fixture
        .job("blocks-after-restart", facts("refs/heads/main"))
        .await;
    assert_eq!(
        restarted.contents("cache-blocks", "v1", &[]).await,
        "abcDEF"
    );
}

#[tokio::test]
async fn v1_streams_ranged_patches_and_keeps_retained_entries() {
    let fixture = Fixture::new().await;
    let job = fixture.job("wire-v1", facts("refs/heads/main")).await;
    let response = job
        .client
        .post(format!("{}/_apis/artifactcache/caches", job.origin))
        .json(&json!({"key":"legacy","version":"v1","cacheSize":6}))
        .send()
        .await
        .unwrap();
    let reservation: serde_json::Value = response.json().await.unwrap();
    let id = reservation["cacheId"].as_u64().unwrap();
    let upload = format!("{}/_apis/artifactcache/caches/{id}", job.origin);
    for (range, data) in [("bytes 3-5/*", "def"), ("bytes 0-2/*", "abc")] {
        assert_eq!(
            job.client
                .patch(&upload)
                .header("content-range", range)
                .body(data)
                .send()
                .await
                .unwrap()
                .status(),
            204
        );
    }
    assert_eq!(
        job.client
            .post(&upload)
            .json(&json!({"size":6}))
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    fixture.delete_upstream().await;
    let response: serde_json::Value = job
        .client
        .get(format!(
            "{}/_apis/artifactcache/cache?keys=legacy&version=v1",
            job.origin
        ))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(response["cacheKey"], "legacy");
    assert_eq!(
        job.client
            .get(response["archiveLocation"].as_str().unwrap())
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap(),
        "abcdef"
    );
    let upstream = fixture.upstream().await;
    assert_eq!(
        upstream["calls"][0]["path"],
        "/legacy-prefix/_apis/artifactcache/caches"
    );
    assert_eq!(
        upstream["calls"][0]["authorization"],
        "Bearer github-wire-v1"
    );
    assert_eq!(
        fixture.report().await.jobs[0].usage,
        JobCacheUsage::MachineHandled(CacheRestoreCounts {
            local_entries: 1,
            github_entries: 0,
            restored_bytes: 6
        })
    );
}
