//! Key rotation, failed refreshes, and fake-key floods use injected time.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use ciproxy_protocol::identifiers::GitRef;
use ciproxy_protocol::jobs::{JobScope, UnknownJobScopeReason};
use jsonwebtoken::Algorithm;
use serde_json::json;
use unimock::{MockFn, Unimock, matching};

use crate::clock::ClockMock;
use crate::error::CacheError;
use crate::runtime_token::keys::{CachedKeys, SigningKeys};
use crate::runtime_token::source::KeySourceMock;
use crate::runtime_token::{RuntimeTokens, TokenVerifier};

use super::support::{NOW, claims, facts, keys, signed_header};

#[tokio::test]
async fn rotated_kid_refreshes_once_then_verifies_the_exact_scope() {
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let source = Arc::new(Unimock::new(
        KeySourceMock::fetch
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                let mut result = keys();
                if count.fetch_add(1, Ordering::SeqCst) > 0 {
                    result.keys[0].common.key_id = Some("rotated".into());
                }
                Ok(result)
            })),
    ));
    let clock = Arc::new(Unimock::new(
        ClockMock::now.each_call(matching!()).returns(NOW),
    ));
    let cache = Arc::new(CachedKeys::new(source, clock.clone()));
    assert!(cache.key("fixture-rsa").await.is_ok());
    let verifier = TokenVerifier::new(cache, clock);
    let token = signed_header(
        &claims(json!([{"Scope":"refs/heads/main","Permission":3}])),
        Algorithm::RS256,
        Some("rotated"),
    );
    for _ in 0..2 {
        assert_eq!(
            verifier.check(&token, &facts()).await,
            JobScope::Checked {
                git_ref: GitRef("refs/heads/main".into()),
                read_refs: vec![GitRef("refs/heads/main".into())],
            }
        );
    }
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn unknown_kid_refreshes_once_and_failed_refreshes_have_a_cooldown() {
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let source = Arc::new(Unimock::new(
        KeySourceMock::fetch
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                match count.fetch_add(1, Ordering::SeqCst) {
                    0 => Ok(keys()),
                    _ => Err(CacheError::Unavailable),
                }
            })),
    ));
    let time = Arc::new(AtomicU64::new(1_000));
    let read = time.clone();
    let clock = Arc::new(Unimock::new(
        ClockMock::now
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| read.load(Ordering::SeqCst))),
    ));
    let cache = CachedKeys::new(source, clock);
    assert!(cache.key("fixture-rsa").await.is_ok());
    assert_eq!(
        cache.key("unknown").await.err(),
        Some(UnknownJobScopeReason::UnknownKey)
    );
    assert_eq!(
        cache.key("another-unknown").await.err(),
        Some(UnknownJobScopeReason::UnknownKey)
    );
    assert!(cache.key("fixture-rsa").await.is_ok());
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    time.store(1_059, Ordering::SeqCst);
    assert_eq!(
        cache.key("unknown").await.err(),
        Some(UnknownJobScopeReason::UnknownKey)
    );
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    time.store(1_060, Ordering::SeqCst);
    assert_eq!(
        cache.key("unknown").await.err(),
        Some(UnknownJobScopeReason::UnknownKey)
    );
    assert_eq!(calls.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn periodic_failure_expires_cached_keys_and_rate_limits_recovery() {
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let source = Arc::new(Unimock::new(
        KeySourceMock::fetch
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                match count.fetch_add(1, Ordering::SeqCst) {
                    1 => Err(CacheError::Unavailable),
                    _ => Ok(keys()),
                }
            })),
    ));
    let time = Arc::new(AtomicU64::new(1_000));
    let read = time.clone();
    let clock = Arc::new(Unimock::new(
        ClockMock::now
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| read.load(Ordering::SeqCst))),
    ));
    let cache = CachedKeys::new(source, clock);
    assert!(cache.key("fixture-rsa").await.is_ok());
    time.store(1_299, Ordering::SeqCst);
    cache.refresh().await;
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    time.store(1_300, Ordering::SeqCst);
    cache.refresh().await;
    assert_eq!(
        cache.key("fixture-rsa").await.err(),
        Some(UnknownJobScopeReason::KeysUnavailable)
    );
    assert_eq!(
        cache.key("unknown").await.err(),
        Some(UnknownJobScopeReason::KeysUnavailable)
    );
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    time.store(1_600, Ordering::SeqCst);
    assert!(cache.key("fixture-rsa").await.is_ok());
    assert_eq!(calls.load(Ordering::SeqCst), 3);
}
