//! Exact-assignment recovery preserves a live proxy and isolates reused job IDs.

use ciproxy_cache_sidecar::session::SessionCredential;
use ciproxy_protocol::cache::{CacheConnection, CacheRestoreCounts, JobCacheUsage};
use ciproxy_protocol::jobs::UnknownJobScopeReason;
use ciproxy_protocol::runtime::CacheRegistration;
use ciproxy_protocol::secret::Secret;

use crate::support::{Fixture, facts};

async fn recover(fixture: &Fixture, registration: &CacheRegistration) -> Option<SessionCredential> {
    fixture
        .client
        .post(format!("{}/sessions/recover", fixture.admin))
        .bearer_auth("admin-secret")
        .json(registration)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap()
}

#[tokio::test]
async fn reconnect_reuses_live_session_and_restart_refreshes_the_existing_proxy() {
    let mut fixture = Fixture::new().await;
    let job = fixture.job("reused", facts("refs/heads/main")).await;
    job.save("kept", "v1", "payload").await;
    assert_eq!(job.contents("kept", "v1", &[]).await, "payload");
    assert_eq!(
        recover(&fixture, &job.registration).await.unwrap().token,
        job.token
    );
    let mut other_assignment = job.registration.clone();
    other_assignment.facts.assignment.runner_id = "72".into();
    assert!(recover(&fixture, &other_assignment).await.is_none());
    fixture.restart().await;
    assert!(recover(&fixture, &job.registration).await.is_none());
    let credential: SessionCredential = fixture
        .client
        .post(format!("{}/sessions", fixture.admin))
        .bearer_auth("admin-secret")
        .json(&job.registration)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_ne!(credential.token, job.token);
    assert_eq!(
        fixture
            .client
            .get(format!(
                "{}/_apis/artifactcache/cache?keys=kept&version=v1",
                fixture.machine
            ))
            .header("x-ciproxy-session", job.token.expose_secret())
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    job.proxy
        .replace_connection(CacheConnection {
            url: Secret::new(&fixture.machine),
            token: credential.token,
        })
        .await;
    fixture.delete_upstream().await;
    assert_eq!(job.contents("kept", "v1", &[]).await, "payload");
    let report = fixture.report().await;
    assert_eq!(
        (report.restores_last_7_days, report.hits_last_7_days),
        (2, 2)
    );
    assert_eq!(report.jobs[0].usage, JobCacheUsage::Github);

    let mut spec = facts("refs/heads/main");
    spec.assignment.runner_id = "72".into();
    let reused = fixture.job("reused", spec).await;
    for path in ["fallback", "close"] {
        assert_eq!(
            fixture
                .client
                .post(format!("{}/sessions/{path}", fixture.admin))
                .bearer_auth("admin-secret")
                .json(&job.registration.facts)
                .send()
                .await
                .unwrap()
                .status(),
            204
        );
    }
    assert_eq!(reused.contents("kept", "v1", &[]).await, "payload");
    assert_eq!(
        recover(&fixture, &reused.registration).await.unwrap().token,
        reused.token
    );
    let report = fixture.report().await;
    let retained = report
        .jobs
        .iter()
        .find(|job| job.assignment.runner_id == "72")
        .unwrap();
    assert_eq!(
        retained.usage,
        JobCacheUsage::MachineHandled(CacheRestoreCounts {
            local_entries: 1,
            github_entries: 0,
            restored_bytes: 7
        })
    );
}

#[tokio::test]
async fn first_unknown_reason_is_retained_per_actual_assignment() {
    let fixture = Fixture::new().await;
    let checked = fixture.job("first-unknown", facts("refs/heads/main")).await;
    assert_eq!(fixture.report().await.unknown_jobs, vec![]);
    let mut spec = facts("refs/heads/main");
    spec.token = Some(Secret::new("invalid"));
    let first = fixture.job("first-unknown", spec.clone()).await;
    spec.token = Some(Secret::new(""));
    let replay = fixture.job("first-unknown", spec.clone()).await;
    spec.assignment.runner_id = "72".into();
    let next = fixture.job("first-unknown", spec).await;
    let report = fixture.report().await;
    assert_eq!(report.jobs.len(), 2);
    assert_eq!(
        report
            .unknown_jobs
            .iter()
            .map(|count| (count.reason, count.jobs))
            .collect::<Vec<_>>(),
        vec![
            (UnknownJobScopeReason::MissingToken, 1),
            (UnknownJobScopeReason::MalformedToken, 1)
        ]
    );
    drop((checked, first, replay, next));
}
