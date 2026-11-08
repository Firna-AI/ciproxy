//! Session reads retain no installation work until an explicit start.

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use ciproxy_api::router::router;
use ciproxy_github::app::webhook::InstallationAction;
use diesel::prelude::*;
use diesel::sql_types::BigInt;
use sha2::{Digest, Sha256};
use tower::ServiceExt;

use super::fixture::{Browser, Fixture};
use super::fixture_io::{CallbackOutcome, json};
use super::link_support::{begin, claims, event, returned, scenario, start};

fn attempts(fixture: &Fixture) -> i64 {
    #[derive(QueryableByName)]
    struct Count {
        #[diesel(sql_type = BigInt)]
        count: i64,
    }
    diesel::sql_query("SELECT count(*) FROM installation_attempts")
        .get_result::<Count>(&mut PgConnection::establish(&fixture.database.url).unwrap())
        .unwrap()
        .count
}

async fn reads(fixture: &Fixture, browser: &Browser, expected: i64) {
    for _ in 0..3 {
        let response = fixture
            .request(Some(browser), "GET", "/session", None)
            .await;
        assert_eq!(response.status(), StatusCode::OK);
        let session = json(response).await;
        assert_eq!(
            attempts(fixture),
            expected,
            "session reads must create no attempt"
        );
        assert_eq!(session["install_url"], "/install");
    }
}

async fn guards(fixture: &Fixture, browser: &Browser) {
    let wrong_origin = "https://attacker.example";
    for (origin, csrf) in [
        (None, Some(browser.csrf.as_str())),
        (Some(fixture.state.config.public_origin.as_str()), None),
        (Some(wrong_origin), Some(browser.csrf.as_str())),
    ] {
        let mut request = Request::builder()
            .method("POST")
            .uri("/installation-links/start")
            .header(header::COOKIE, format!("__Host-ciproxy={}", browser.cookie));
        if let Some(origin) = origin {
            request = request.header(header::ORIGIN, origin);
        }
        if let Some(csrf) = csrf {
            request = request.header("x-ciproxy-csrf", csrf);
        }
        let response = router(fixture.state.clone())
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
    let response = fixture
        .request(Some(browser), "GET", "/installation-links/start", None)
        .await;
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(attempts(fixture), 0);
}

#[tokio::test]
async fn repeated_session_reads_create_no_attempt_and_explicit_start_keeps_the_session_purpose() {
    let (fixture, _server, admin, added) = scenario().await;
    reads(&fixture, &admin, 0).await;
    guards(&fixture, &admin).await;
    let state = start(&fixture, &admin).await;
    let attempt = fixture
        .database
        .store
        .installation_attempt(&Sha256::digest(state.as_bytes()))
        .unwrap()
        .unwrap();
    assert!(attempt.direct);
    assert_eq!(attempt.requester_id, 7);
    assert_eq!(attempts(&fixture), 1);
    reads(&fixture, &admin, 1).await;

    let state = begin(&fixture, &admin).await;
    assert!(fixture.database.store.has_pending_add(7).unwrap());
    let continuing = returned(&fixture, &state, CallbackOutcome::Accepted)
        .await
        .unwrap();
    reads(&fixture, &continuing, 2).await;
    let state = start(&fixture, &continuing).await;
    let attempt = fixture
        .database
        .store
        .installation_attempt(&Sha256::digest(state.as_bytes()))
        .unwrap()
        .unwrap();
    assert!(!attempt.direct);
    assert_eq!(attempt.requester_id, 7);
    assert_eq!(attempts(&fixture), 3);
    fixture
        .app
        .installation(event(&added, 7, InstallationAction::Created))
        .await
        .unwrap();
    assert_eq!(
        claims(&fixture, &continuing).await[0]["state"],
        "linked_inactive"
    );
    let state = start(&fixture, &continuing).await;
    assert!(
        fixture
            .database
            .store
            .installation_attempt(&Sha256::digest(state.as_bytes()))
            .unwrap()
            .unwrap()
            .direct
    );
    assert_eq!(attempts(&fixture), 4);
}
