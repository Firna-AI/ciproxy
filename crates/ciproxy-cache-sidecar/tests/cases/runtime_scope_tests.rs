//! The authenticated machine applies unknown-token policy to real cache calls.

use ciproxy_cache_sidecar::session::SessionCredential;
use ciproxy_protocol::cache::UnknownJobCount;
use ciproxy_protocol::identifiers::{GitRef, JobEvent};
use ciproxy_protocol::jobs::{JobScope, UnknownJobScopeReason};
use ciproxy_protocol::runtime::CachePermissions;
use ciproxy_protocol::secret::Secret;
use serde_json::json;

use crate::support::tokens::signed;
use crate::support::{Fixture, V2, facts};

#[tokio::test]
async fn unknown_jobs_read_default_only_and_never_restore_their_private_saves() {
    let mut fixture = Fixture::new().await;
    let main = fixture.job("main", facts("refs/heads/main")).await;
    main.save("main-entry", "v1", "main").await;
    let feature = fixture.job("feature", facts("refs/heads/feature")).await;
    feature.save("feature-entry", "v1", "feature").await;
    let mut spec = facts("refs/heads/main");
    spec.token = Some(Secret::new("invalid-runtime-token"));
    let unknown = fixture.job("unknown", spec.clone()).await;
    assert_eq!(
        unknown.scope,
        JobScope::Unknown {
            reason: UnknownJobScopeReason::MalformedToken
        }
    );
    unknown.save("private-entry", "v1", "private").await;
    assert_eq!(
        unknown.contents("private-entry", "v1", &[]).await,
        "private"
    );
    let before = fixture.report().await;
    assert_eq!(
        before.unknown_jobs,
        vec![UnknownJobCount {
            reason: UnknownJobScopeReason::MalformedToken,
            jobs: 1
        }]
    );
    assert_eq!(before.hits_last_7_days, 0);
    assert!(before.used_bytes >= 18);
    fixture.delete_upstream().await;
    assert_eq!(unknown.contents("main-entry", "v1", &[]).await, "main");
    assert_eq!(
        unknown.lookup("feature-entry", "v1", &[]).await["ok"],
        false
    );
    assert_eq!(
        unknown.lookup("private-entry", "v1", &[]).await["ok"],
        false
    );
    spec.token = Some(Secret::new(""));
    let same_id = fixture.job("unknown", spec).await;
    assert_eq!(
        same_id.scope,
        JobScope::Unknown {
            reason: UnknownJobScopeReason::MissingToken
        }
    );
    assert_eq!(fixture.report().await.unknown_jobs, before.unknown_jobs);
    same_id.save("private-entry", "v1", "new-private").await;
    assert_eq!(fixture.report().await.used_bytes, before.used_bytes + 11);
    fixture.delete_upstream().await;
    assert_eq!(
        same_id.lookup("private-entry", "v1", &[]).await["ok"],
        false
    );
    assert_eq!(main.lookup("private-entry", "v1", &[]).await["ok"], false);
    fixture.restart().await;
    assert_eq!(fixture.report().await.unknown_jobs, before.unknown_jobs);
}

#[tokio::test]
async fn read_only_and_pr_target_tokens_are_unknown_and_keep_upstream_saves() {
    let fixture = Fixture::new().await;
    for (id, event, access, permissions, saves) in [
        (
            "target",
            "pull_request_target",
            json!([{"Scope":"refs/heads/main","Permission":1}]),
            CachePermissions {
                read: true,
                write: true,
            },
            true,
        ),
        (
            "read",
            "push",
            json!([{"Scope":"refs/heads/feature","Permission":1},{"Scope":"refs/heads/main","Permission":1}]),
            CachePermissions {
                read: true,
                write: false,
            },
            false,
        ),
    ] {
        let mut spec = facts("refs/heads/main");
        spec.event = JobEvent(event.into());
        spec.token = Some(signed(
            &json!({"iss":"https://token.actions.githubusercontent.com","nbf":999_000,"exp":2_000_000,
            "repository_id":"10","run_id":"31","runner_id":"71","job_id":id,"ac":access.to_string()}),
        ));
        let job = fixture.job_with(id, spec, permissions, false).await;
        assert_eq!(
            job.scope,
            JobScope::Unknown {
                reason: UnknownJobScopeReason::NoWriteScope
            }
        );
        assert_eq!(
            job.rpc("CreateCacheEntry", json!({"key":id,"version":"v1"}))
                .await["ok"],
            saves
        );
    }
    assert_eq!(
        fixture.upstream().await["calls"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        fixture.report().await.unknown_jobs,
        vec![UnknownJobCount {
            reason: UnknownJobScopeReason::NoWriteScope,
            jobs: 2
        }]
    );
}

#[tokio::test]
async fn tag_named_main_keeps_raw_scope_and_cannot_publish_default_entries() {
    let fixture = Fixture::new().await;
    let tag = fixture.job("tag", facts("refs/heads/refs/tags/main")).await;
    assert_eq!(
        tag.scope,
        JobScope::Checked {
            git_ref: GitRef("refs/heads/refs/tags/main".into()),
            read_refs: vec![
                GitRef("refs/heads/refs/tags/main".into()),
                GitRef("refs/heads/main".into())
            ],
        }
    );
    tag.save("tag-only", "v1", "tag").await;
    fixture.delete_upstream().await;
    let main = fixture.job("main", facts("refs/heads/main")).await;
    assert_eq!(main.lookup("tag-only", "v1", &[]).await["ok"], false);
    let later_tag = fixture
        .job("later-tag", facts("refs/heads/refs/tags/main"))
        .await;
    assert_eq!(later_tag.contents("tag-only", "v1", &[]).await, "tag");
}

#[tokio::test]
async fn missing_token_registers_unknown_and_job_listener_cannot_read_checked_scope() {
    let fixture = Fixture::new().await;
    let mut spec = facts("refs/heads/main");
    spec.assignment.job_id = None;
    let registration = json!({"job_id":"missing","facts":spec.facts,"permissions":{"read":true,"write":true},
        "cache_url":"http://unused.invalid","results_url":"http://unused.invalid","proxy_url":"http://unused.invalid"});
    let response = fixture
        .client
        .post(format!("{}/sessions", fixture.admin))
        .bearer_auth("admin-secret")
        .json(&registration)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let result: SessionCredential = response.json().await.unwrap();
    assert_eq!(
        result.scope,
        JobScope::Unknown {
            reason: UnknownJobScopeReason::MissingToken
        }
    );
    assert_eq!(
        fixture
            .client
            .post(format!("{}/sessions", fixture.machine))
            .json(&registration)
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    assert_eq!(
        fixture
            .client
            .get(format!("{}/report", fixture.admin))
            .bearer_auth(result.token.expose_secret())
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    assert_eq!(
        fixture
            .client
            .post(format!("{}{V2}CreateCacheEntry", fixture.machine))
            .json(&json!({"key":"no-session","version":"v1"}))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
}
