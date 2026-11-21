//! Exact selected target survives concurrent webhook replay and maintenance.

use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use ciproxy_api::maintenance::{ApiMaintenance, Maintenance};
use ciproxy_api::router::router;
use ciproxy_github::app::InstallationOutcome;
use ciproxy_github::app::webhook::InstallationAction;
use diesel::prelude::*;
use diesel::sql_types::BigInt;
use serde_json::json;
use tower::ServiceExt;

use super::fixture_io::{CallbackOutcome, json};
use super::link_support::{begin, claims, event, other_workspace, pending, returned, scenario};
use super::ports_gate::ReadGate;

#[derive(QueryableByName)]
struct Waiters {
    #[diesel(sql_type=BigInt)]
    count: i64,
}

async fn wait_for_two_lease_waiters(connection: &mut PgConnection) {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let waiting=diesel::sql_query("SELECT count(*) AS count FROM pg_locks l JOIN pg_stat_activity a ON a.pid=l.pid WHERE a.datname=current_database() AND l.locktype='advisory' AND NOT l.granted")
            .get_result::<Waiters>(connection).unwrap();
        if waiting.count >= 2 {
            return;
        }
        if Instant::now() >= deadline {
            panic!("the webhook and maintenance did not reach their organization lease");
        }
        tokio::task::yield_now().await;
    }
}

#[tokio::test]
async fn selected_claim_webhook_and_maintenance_serialize_without_changing_its_target() {
    let (fixture, _server, admin, added) = scenario().await;
    let other = other_workspace(&fixture);
    let state = begin(&fixture, &admin).await;
    let id = pending(
        fixture
            .app
            .installation(event(&added, 7, InstallationAction::Created))
            .await
            .unwrap(),
    );
    let current = returned(&fixture, &state, CallbackOutcome::Accepted)
        .await
        .unwrap();
    assert_eq!(
        claims(&fixture, &current).await[0]["state"],
        "choose_workspace"
    );
    let gate = Arc::new(ReadGate::default());
    *fixture.remote.read_gate.lock().unwrap() = Some(gate.clone());
    let app_state = fixture.state.clone();
    let request = Request::builder()
        .method("PUT")
        .uri(format!("/installation-links/{id}/workspace"))
        .header(header::COOKIE, format!("__Host-ciproxy={}", current.cookie))
        .header(header::ORIGIN, &app_state.config.public_origin)
        .header("x-ciproxy-csrf", &current.csrf)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({"workspace_id":other.id}).to_string()))
        .unwrap();
    let choosing = tokio::spawn(async move { router(app_state).oneshot(request).await.unwrap() });
    gate.entered.notified().await;
    let app = fixture.app.clone();
    let delayed = tokio::spawn(async move {
        app.installation(event(&added, 7, InstallationAction::Created))
            .await
    });
    let maintenance = Maintenance {
        state: fixture.state.clone(),
    };
    let reconciling = tokio::spawn(async move { maintenance.reconcile().await });
    let mut observer = PgConnection::establish(&fixture.database.url).unwrap();
    wait_for_two_lease_waiters(&mut observer).await;
    gate.release.notify_one();
    let (response, delivered, reconciled) =
        tokio::time::timeout(Duration::from_secs(20), async move {
            (
                choosing.await.unwrap(),
                delayed.await.unwrap(),
                reconciling.await.unwrap(),
            )
        })
        .await
        .expect("installation lease participants did not finish");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(json(response).await["workspace_id"], other.id.to_string());
    assert!(
        matches!(delivered,Ok(InstallationOutcome::Organization(ref saved)) if saved.workspace_id==other.id && saved.installation_id==1066 && saved.github_id==106)
    );
    assert!(matches!(reconciled, Ok(())));
    let claim = fixture
        .database
        .store
        .installation_claim(id)
        .unwrap()
        .unwrap();
    assert_eq!(claim.workspace_id, Some(other.id));
    assert_eq!(
        fixture
            .database
            .store
            .organizations_by_github_id(106)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        fixture
            .database
            .store
            .organization_by_installation_id(1066)
            .unwrap()
            .unwrap()
            .workspace_id,
        other.id
    );
}
