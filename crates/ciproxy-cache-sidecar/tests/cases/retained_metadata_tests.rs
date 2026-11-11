//! Retained writer facts, metadata enrichment, and legacy provenance.

use crate::support::restore::contents;
use crate::support::scoped::Scoped;
use crate::support::{Fixture, facts};

#[tokio::test]
async fn retained_writer_without_creation_metadata_beats_lower_ref_upstream() {
    let mut fixture = Fixture::new().await;
    let writer = fixture.job("writer", facts("refs/heads/feature")).await;
    writer.save("deps-retained", "v1", "accepted-feature").await;
    let peer = Scoped::start(&mut fixture).await;
    peer.add("refs/heads/main", "deps-exact", "default", 200)
        .await;
    fixture.restart().await;
    peer.configure(&fixture).await;
    let feature = fixture
        .job("writer-after-restart", facts("refs/heads/feature"))
        .await;
    assert_eq!(
        contents(&feature, false, "deps-exact").await,
        "accepted-feature"
    );
}

#[tokio::test]
async fn upstream_creation_time_orders_prefixes_and_enrichment_survives_restart() {
    let mut fixture = Fixture::new().await;
    let writer = fixture
        .job("writer-without-metadata", facts("refs/heads/feature"))
        .await;
    writer.save("deps-old", "v1", "accepted-old").await;
    let stale = writer.lookup("deps-old", "v1", &[]).await;
    let peer = Scoped::start(&mut fixture).await;
    peer.add("refs/heads/feature", "deps-old", "accepted-old", 100)
        .await;
    peer.add("refs/heads/feature", "deps-new", "newer-upstream", 200)
        .await;
    let feature = fixture.job("reader", facts("refs/heads/feature")).await;
    assert_eq!(
        contents(&feature, false, "deps-exact").await,
        "newer-upstream"
    );
    let old = fixture
        .entries()
        .await
        .into_iter()
        .find(|entry| entry.key == "deps-old")
        .unwrap();
    assert_eq!(
        old.source.as_ref().map(|source| source.id.as_str()),
        Some("100")
    );
    assert_eq!(old.accessed, 1, "metadata enrichment does not touch LRU");
    assert_eq!(
        writer
            .client
            .get(stale["signedDownloadUrl"].as_str().unwrap())
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap(),
        "accepted-old"
    );
    let touched = fixture
        .entries()
        .await
        .into_iter()
        .find(|entry| entry.key == "deps-old")
        .unwrap();
    assert_eq!(
        touched.source.as_ref().map(|source| source.id.as_str()),
        Some("100"),
        "an older download ticket cannot erase the learned identity"
    );
    peer.remove("refs/heads/feature").await;
    fixture.restart().await;
    peer.configure(&fixture).await;
    let next = fixture.job("later", facts("refs/heads/feature")).await;
    assert_eq!(contents(&next, false, "deps-exact").await, "newer-upstream");
}

#[tokio::test]
async fn legacy_reader_fill_keeps_bytes_without_becoming_source_authority() {
    let mut fixture = Fixture::new().await;
    fixture.seed("legacy", "v1", "bGVnYWN5LWJ5dGVz").await;
    let first = fixture
        .job("legacy-reader", facts("refs/heads/feature"))
        .await;
    assert_eq!(first.contents("legacy", "v1", &[]).await, "legacy-bytes");
    let peer = Scoped::start(&mut fixture).await;
    peer.add("refs/heads/main", "legacy", "current-default", 200)
        .await;
    let next = fixture
        .job("current-reader", facts("refs/heads/feature"))
        .await;
    assert_eq!(contents(&next, false, "legacy").await, "current-default");
    let unresolved = fixture
        .entries()
        .await
        .into_iter()
        .find(|entry| entry.source.is_none())
        .unwrap();
    assert_eq!(unresolved.key, "legacy");
    peer.remove("refs/heads/main").await;
    fixture.restart().await;
    peer.configure(&fixture).await;
    let retained = fixture
        .job("legacy-retained", facts("refs/heads/feature"))
        .await;
    assert_eq!(
        contents(&retained, false, "legacy").await,
        "current-default"
    );
    assert_eq!(
        fixture
            .entries()
            .await
            .iter()
            .filter(|entry| entry.source.is_none())
            .count(),
        1
    );
}
