//! Protect branch precedence, repository separation, and hostile PR writes.

use ciproxy_cache_sidecar::session::Permissions;
use ciproxy_protocol::identifiers::{GitRef, JobEvent, RepositoryId};
use ciproxy_protocol::jobs::Repository;
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::support::{Fixture, facts};

#[tokio::test]
async fn accepted_branch_save_supersedes_reader_fill_and_survives_restart() {
    let mut fixture = Fixture::new().await;
    fixture.seed("borrowed", "v1", "ZGVmYXVsdC12YWx1ZQ==").await;
    let branch = fixture
        .job("borrowed-feature", facts("refs/heads/feature"))
        .await;
    assert_eq!(
        branch.contents("borrowed", "v1", &[]).await,
        "default-value"
    );
    fixture.delete_upstream().await;
    assert_eq!(
        branch.contents("borrowed", "v1", &[]).await,
        "default-value"
    );
    fixture.restart().await;
    let branch = fixture
        .job("borrowed-after-restart", facts("refs/heads/feature"))
        .await;
    assert_eq!(
        branch.contents("borrowed", "v1", &[]).await,
        "default-value"
    );
    branch.save("borrowed", "v1", "feature-value").await;
    let state = fixture.upstream().await;
    assert_eq!(
        state["entries"][0]["sha256"],
        "6eef5a39b126b5ed818032eeee26c5ce1743dabb8501f95e97ca57ac8c337f47"
    );
    fixture.delete_upstream().await;
    assert_eq!(
        branch.contents("borrowed", "v1", &[]).await,
        "feature-value"
    );
    branch.save("borrowed", "v1", "second-feature-value").await;
    assert_eq!(
        branch.contents("borrowed", "v1", &[]).await,
        "feature-value"
    );
    fixture.restart().await;
    let later = fixture
        .job("saved-after-restart", facts("refs/heads/feature"))
        .await;
    assert_eq!(later.contents("borrowed", "v1", &[]).await, "feature-value");
}

#[tokio::test]
async fn scopes_keep_own_prefix_before_default_exact_and_isolate_prs() {
    let fixture = Fixture::new().await;
    let main = fixture.job("scope-main", facts("refs/heads/main")).await;
    main.save("cache-exact", "v1", "main").await;
    main.save("cache-exact", "v1", "overwritten-upstream").await;
    let branch = fixture
        .job("scope-branch", facts("refs/heads/feature"))
        .await;
    branch.save("cache-prefix-old", "v1", "old").await;
    branch.save("cache-prefix-new", "v1", "new").await;
    fixture.options(json!({"lookup":false})).await;
    assert_eq!(
        branch
            .contents("cache-exact", "v1", &["cache-prefix-"])
            .await,
        "new"
    );
    assert_eq!(branch.contents("cache-prefix-old", "v1", &[]).await, "old");
    assert_eq!(main.contents("cache-exact", "v1", &[]).await, "main");
    assert_eq!(
        main.lookup("cache-prefix-new", "v1", &[]).await["ok"],
        false
    );
    assert_eq!(branch.lookup("cache-exact", "v2", &[]).await["ok"], false);
    let mut pr_facts = facts("refs/pull/7/merge");
    pr_facts.event = JobEvent("pull_request".into());
    pr_facts.base_ref = Some(GitRef("refs/heads/feature".into()));
    let pr = fixture.job("scope-pr7", pr_facts.clone()).await;
    assert_eq!(pr.contents("cache-prefix-new", "v1", &[]).await, "new");
    pr.save("private", "v1", "pr7").await;
    assert_eq!(pr.contents("private", "v1", &[]).await, "pr7");
    pr_facts.git_ref = GitRef("refs/pull/8/merge".into());
    let other = fixture.job("scope-pr8", pr_facts).await;
    assert_eq!(other.lookup("private", "v1", &[]).await["ok"], false);
    assert_eq!(main.lookup("private", "v1", &[]).await["ok"], false);
    let mut repository = facts("refs/heads/main");
    repository.repository.id = RepositoryId("99".into());
    let separate = fixture.job("other-repo", repository).await;
    assert_eq!(separate.lookup("cache-exact", "v1", &[]).await["ok"], false);
}

#[tokio::test]
async fn unsafe_pr_and_fork_writes_never_reach_upstream_even_in_fallback() {
    let fixture = Fixture::new().await;
    for (index, event, fork, down) in [
        (0, "pull_request_target", false, false),
        (1, "pull_request_target", false, true),
        (2, "pull_request", false, false),
        (3, "push", true, false),
    ] {
        let mut job_facts = facts("refs/heads/main");
        job_facts.event = JobEvent(event.into());
        if fork {
            job_facts.head_repository = Some(Repository {
                id: RepositoryId("20".into()),
                full_name: "fork/repo".into(),
            });
        }
        let job = fixture
            .job_with(
                &format!("unsafe-{index}"),
                job_facts,
                Permissions {
                    read: true,
                    write: true,
                },
                down,
            )
            .await;
        for operation in ["CreateCacheEntry", "FinalizeCacheEntryUpload"] {
            assert_eq!(
                job.rpc(
                    operation,
                    json!({"key":"poison","version":"v1","sizeBytes":"4"})
                )
                .await["ok"],
                false
            );
            for prefix in ["/other/../", "/other/%2e%2e/", "//"] {
                let target =
                    format!("{prefix}twirp/github.actions.results.api.v1.CacheService/{operation}");
                let origin = url::Url::parse(&job.origin).unwrap();
                let mut stream = TcpStream::connect(origin.socket_addrs(|| None).unwrap()[0])
                    .await
                    .unwrap();
                let body = "{\"key\":\"poison\",\"version\":\"v1\",\"sizeBytes\":\"4\"}";
                stream.write_all(format!("POST {target} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}", body.len()).as_bytes()).await.unwrap();
                let mut response = String::new();
                stream.read_to_string(&mut response).await.unwrap();
                assert!(response.contains("\"ok\":false"), "{target}: {response}");
            }
        }
    }
    assert_eq!(fixture.upstream().await["calls"], json!([]));
    assert_eq!(fixture.report().await.used_bytes, 0);
}
