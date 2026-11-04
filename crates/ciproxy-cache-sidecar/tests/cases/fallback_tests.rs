//! Protect disk and HTTP fallback without accepting guest authority.

use ciproxy_cache_sidecar::session::Permissions;
use serde_json::json;

use crate::support::{Fixture, V2, facts};

#[tokio::test]
async fn disk_faults_and_machine_outage_keep_github_cache_and_results_usable() {
    let fixture = Fixture::with_disk(true, 1024 * 1024).await;
    let job = fixture.job("disk-fault", facts("refs/heads/main")).await;
    job.save("upstream-only", "v1", "github").await;
    assert_eq!(job.contents("upstream-only", "v1", &[]).await, "github");
    let direct = fixture
        .job_with(
            "machine-down",
            facts("refs/heads/main"),
            Permissions {
                read: true,
                write: true,
            },
            true,
        )
        .await;
    assert_eq!(direct.contents("upstream-only", "v1", &[]).await, "github");
    direct.save("fallback-save", "v1", "saved").await;
    let response = direct
        .client
        .post(format!(
            "{}/twirp/github.actions.results.api.v1.ArtifactService/CreateArtifact?x=one",
            direct.origin
        ))
        .header("x-example", "value")
        .body("artifact-log-payload")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 202);
    assert_eq!(response.headers()["x-fixture"], "results-pass-through");
    assert_eq!(response.text().await.unwrap(), "artifact-log-payload");
    let response = direct
        .client
        .post(format!(
            "{}//untrusted.invalid/results?query=kept",
            direct.origin
        ))
        .body("authority-stays-github")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 202);
    assert_eq!(response.text().await.unwrap(), "authority-stays-github");
    fixture.options(json!({"failure":true})).await;
    assert_eq!(direct.lookup("upstream-only", "v1", &[]).await["ok"], false);
    assert_eq!(job.lookup("absent", "v1", &[]).await["ok"], false);
}

#[tokio::test]
async fn session_auth_and_trusted_permissions_block_guest_authority() {
    let fixture = Fixture::new().await;
    let writer = fixture.job("auth-writer", facts("refs/heads/main")).await;
    writer.save("protected", "v1", "trusted").await;
    let denied = fixture
        .job_with(
            "no-permissions",
            facts("refs/heads/main"),
            Permissions::default(),
            false,
        )
        .await;
    assert_eq!(denied.lookup("protected", "v1", &[]).await["ok"], false);
    assert_eq!(
        denied
            .rpc(
                "CreateCacheEntry",
                json!({"key":"bad","version":"v1","metadata":{"scope":"refs/heads/main"}})
            )
            .await["ok"],
        false
    );
    assert_eq!(
        fixture
            .client
            .post(format!("{}/clear", fixture.admin))
            .bearer_auth(writer.token.expose_secret())
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
            .json(&json!({"key":"bad","version":"v1"}))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    let lookup = writer.lookup("protected", "v1", &[]).await;
    let path = url::Url::parse(lookup["signedDownloadUrl"].as_str().unwrap())
        .unwrap()
        .path()
        .to_owned();
    let another = fixture
        .job("unrelated-session", facts("refs/heads/main"))
        .await;
    assert_eq!(
        another
            .client
            .get(format!("{}{}", another.origin, path))
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    assert_eq!(writer.contents("protected", "v1", &[]).await, "trusted");
    assert_eq!(
        fixture
            .client
            .delete(format!(
                "{}/sessions/{}",
                fixture.admin,
                writer.token.expose_secret()
            ))
            .bearer_auth("admin-secret")
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    assert_eq!(
        fixture
            .client
            .get(format!(
                "{}/_apis/artifactcache/cache?keys=protected&version=v1",
                fixture.machine
            ))
            .header("x-ciproxy-session", writer.token.expose_secret())
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
}
