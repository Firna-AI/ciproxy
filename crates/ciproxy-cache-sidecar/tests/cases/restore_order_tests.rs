//! Cross-tier restore order and retained source identities through public HTTP.

use crate::support::restore::{contents, lookup};
use crate::support::scoped::Scoped;
use crate::support::{Fixture, facts};

async fn feature_exact_over_default_prefix(legacy: bool) {
    let mut fixture = Fixture::new().await;
    let peer = Scoped::start(&mut fixture).await;
    peer.add("refs/heads/main", "deps-old", "old-default", 100)
        .await;
    let main = fixture
        .job("default-reader", facts("refs/heads/main"))
        .await;
    assert_eq!(contents(&main, legacy, "deps-old").await, "old-default");
    let feature = fixture
        .job("feature-reader", facts("refs/heads/feature"))
        .await;
    peer.add("refs/heads/feature", "deps-exact", "feature-exact", 200)
        .await;
    let entry = lookup(&feature, legacy, "deps-exact").await;
    let field = if legacy { "cacheKey" } else { "matchedKey" };
    assert_eq!(entry[field], "deps-exact");
    assert_eq!(
        contents(&feature, legacy, "deps-exact").await,
        "feature-exact"
    );
}

#[tokio::test]
async fn v1_feature_exact_precedes_local_default_prefix() {
    feature_exact_over_default_prefix(true).await;
}

#[tokio::test]
async fn v2_feature_exact_precedes_local_default_prefix() {
    feature_exact_over_default_prefix(false).await;
}

#[tokio::test]
async fn retained_feature_fill_precedes_upstream_default_exact_after_restart() {
    let mut fixture = Fixture::new().await;
    let peer = Scoped::start(&mut fixture).await;
    peer.add(
        "refs/heads/feature",
        "deps-retained",
        "retained-feature",
        100,
    )
    .await;
    let first = fixture.job("first", facts("refs/heads/feature")).await;
    assert_eq!(
        contents(&first, false, "deps-retained").await,
        "retained-feature"
    );
    peer.remove("refs/heads/feature").await;
    peer.add("refs/heads/main", "deps-exact", "default-exact", 200)
        .await;
    fixture.restart().await;
    peer.configure(&fixture).await;
    let next = fixture.job("next", facts("refs/heads/feature")).await;
    for legacy in [true, false] {
        assert_eq!(
            contents(&next, legacy, "deps-exact").await,
            "retained-feature"
        );
    }
}

#[tokio::test]
async fn borrowed_fill_keeps_default_source_when_feature_entry_arrives() {
    let mut fixture = Fixture::new().await;
    let peer = Scoped::start(&mut fixture).await;
    peer.add("refs/heads/main", "deps-exact", "borrowed-default", 100)
        .await;
    let first = fixture
        .job("borrowed-reader", facts("refs/heads/feature"))
        .await;
    assert_eq!(
        contents(&first, false, "deps-exact").await,
        "borrowed-default"
    );
    peer.add("refs/heads/feature", "deps-prefix", "new-feature", 200)
        .await;
    fixture.restart().await;
    peer.configure(&fixture).await;
    let feature = fixture
        .job("borrowed-after-restart", facts("refs/heads/feature"))
        .await;
    assert_eq!(contents(&feature, false, "deps-exact").await, "new-feature");
    let entries = fixture.entries().await;
    let borrowed = entries
        .iter()
        .find(|entry| entry.key == "deps-exact")
        .unwrap();
    assert_eq!(
        borrowed.source.as_ref().unwrap().git_ref.0,
        "refs/heads/main"
    );
}

#[tokio::test]
async fn ambiguous_and_changed_runtime_sources_never_publish_guessed_fills() {
    let mut fixture = Fixture::new().await;
    let peer = Scoped::start(&mut fixture).await;
    peer.add("refs/heads/main", "same-key", "default", 100)
        .await;
    peer.arrive_during_lookup("refs/heads/feature", "same-key", "feature", 200)
        .await;
    let feature = fixture
        .job("source-race", facts("refs/heads/feature"))
        .await;
    assert_eq!(contents(&feature, false, "same-key").await, "feature");
    assert_eq!(
        fixture.entries().await.len(),
        0,
        "metadata changed during lookup"
    );
    assert_eq!(contents(&feature, false, "same-key").await, "feature");
    assert_eq!(
        fixture.entries().await.len(),
        0,
        "same key exists in two refs"
    );
    peer.remove("refs/heads/feature").await;
    peer.override_scope("refs/heads/feature").await;
    assert_eq!(contents(&feature, true, "same-key").await, "default");
    assert_eq!(
        fixture.entries().await.len(),
        0,
        "v1 scope disagrees with hosted source"
    );
}

#[tokio::test]
async fn unavailable_metadata_uses_github_without_erasing_retained_source() {
    let mut fixture = Fixture::new().await;
    let peer = Scoped::start(&mut fixture).await;
    peer.add("refs/heads/feature", "deps-retained", "retained", 100)
        .await;
    let feature = fixture
        .job("metadata-outage", facts("refs/heads/feature"))
        .await;
    assert_eq!(contents(&feature, false, "deps-retained").await, "retained");
    peer.remove("refs/heads/feature").await;
    peer.add("refs/heads/main", "deps-exact", "direct-default", 200)
        .await;
    peer.unavailable();
    assert_eq!(
        contents(&feature, false, "deps-exact").await,
        "direct-default"
    );
    assert_eq!(fixture.entries().await.len(), 1);
    peer.available();
    assert_eq!(contents(&feature, false, "deps-exact").await, "retained");
}
