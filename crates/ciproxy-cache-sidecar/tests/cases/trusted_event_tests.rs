//! Trusted events govern shared saves and publication of reader fills.

use ciproxy_cache_sidecar::session::Permissions;
use ciproxy_protocol::identifiers::JobEvent;
use serde_json::{Value, json};

use crate::support::{Fixture, V2, facts};

const SHARED_WRITERS: [(&str, bool); 12] = [
    ("push", true),
    ("workflow_dispatch", true),
    ("repository_dispatch", true),
    ("delete", true),
    ("registry_package", true),
    ("page_build", true),
    ("schedule", true),
    ("pull_request", false),
    ("pull_request_target", false),
    ("issue_comment", false),
    ("workflow_run", false),
    ("unknown_event", false),
];

#[tokio::test]
async fn shared_saves_accept_only_trusted_events_at_machine_and_proxy() {
    let fixture = Fixture::new().await;
    for (event, shared) in SHARED_WRITERS {
        let mut job_facts = facts("refs/heads/main");
        job_facts.event = JobEvent(event.into());
        let writer = fixture.job(&format!("writer-{event}"), job_facts).await;
        for (route, origin) in [("proxy", &writer.origin), ("machine", &fixture.machine)] {
            let key = format!("save-{event}-{route}");
            let calls = fixture.upstream().await["calls"].as_array().unwrap().len();
            let reserve = writer
                .client
                .post(format!("{origin}{V2}CreateCacheEntry"))
                .header("x-ciproxy-session", writer.token.expose_secret())
                .json(&json!({"key":key,"version":"v1"}))
                .send()
                .await
                .unwrap();
            assert_eq!(reserve.status(), 200, "{event} through {route}");
            let reserve: Value = reserve.json().await.unwrap();
            assert_eq!(reserve["ok"], shared, "{event} through {route}");
            let uploaded = if let Some(url) = reserve["signedUploadUrl"].as_str() {
                Some(
                    writer
                        .client
                        .put(url)
                        .body("payload")
                        .send()
                        .await
                        .unwrap()
                        .status()
                        .as_u16(),
                )
            } else {
                None
            };
            assert_eq!(uploaded, shared.then_some(201), "{event} through {route}");
            let finalized: Value = writer
                .client
                .post(format!("{origin}{V2}FinalizeCacheEntryUpload"))
                .header("x-ciproxy-session", writer.token.expose_secret())
                .json(&json!({"key":key,"version":"v1","sizeBytes":"7"}))
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            assert_eq!(finalized["ok"], shared, "{event} through {route}");
            assert_eq!(
                fixture.upstream().await["calls"].as_array().unwrap().len() - calls,
                if shared { 3 } else { 0 },
                "{event} upstream writes through {route}"
            );
            fixture.delete_upstream().await;
            let observer = fixture
                .job(
                    &format!("observer-{event}-{route}"),
                    facts("refs/heads/main"),
                )
                .await;
            assert_eq!(
                observer.lookup(&key, "v1", &[]).await["ok"],
                shared,
                "{event} shared save through {route}"
            );
        }
    }
}

#[tokio::test]
async fn reader_fills_stay_private_unless_the_event_is_trusted() {
    let fixture = Fixture::new().await;
    for (event, shared) in SHARED_WRITERS {
        let key = format!("fill-{event}");
        fixture.seed(&key, "v1", "cGF5bG9hZA==").await;
        let mut job_facts = facts("refs/heads/main");
        job_facts.event = JobEvent(event.into());
        let reader = fixture
            .job_with(
                &format!("reader-{event}"),
                job_facts,
                Permissions {
                    read: true,
                    write: false,
                },
                false,
            )
            .await;
        assert_eq!(reader.contents(&key, "v1", &[]).await, "payload", "{event}");
        fixture.delete_upstream().await;
        assert_eq!(reader.contents(&key, "v1", &[]).await, "payload", "{event}");
        let observer = fixture
            .job(&format!("fill-observer-{event}"), facts("refs/heads/main"))
            .await;
        assert_eq!(
            observer.lookup(&key, "v1", &[]).await["ok"],
            shared,
            "{event} shared reader fill"
        );
    }
}
