//! Organization source dates, complete repository counts, and retained adder facts.

use axum::http::StatusCode;
use ciproxy_github::app::webhook::InstallationAction;
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::github::Repository;
use ciproxy_store_interface::organizations::RepositoryAccess;
use diesel::prelude::*;
use serde_json::Value;

use super::fixture_io::{CallbackOutcome, json};
use super::link_support::{begin, event, returned, scenario};

#[tokio::test]
async fn m20_organization_facts_preserve_dates_counts_and_adder_after_reinstall() {
    let (fixture, server, admin, added) = scenario().await;
    let path = fixture.path("/organizations");
    let initial = json(fixture.request(Some(&admin), "GET", &path, None).await).await;
    assert_eq!(initial[0]["repositories"], Value::Null);
    let state = begin(&fixture, &admin).await;
    fixture
        .app
        .installation(event(&added, 7, InstallationAction::Created))
        .await
        .unwrap();
    let current = returned(&fixture, &state, CallbackOutcome::Accepted)
        .await
        .unwrap();
    let added = fixture
        .database
        .store
        .organization_by_installation_id(added.installation_id)
        .unwrap()
        .unwrap();
    fixture
        .database
        .store
        .replace_repositories(
            fixture.workspace,
            added.id,
            &[
                repository(10, true),
                repository(11, true),
                repository(12, false),
            ],
        )
        .unwrap();
    fixture
        .database
        .store
        .save_repository_access(
            fixture.workspace,
            added.id,
            &RepositoryAccess::Selected(vec![10, 12, 999]),
        )
        .unwrap();
    let body = json(fixture.request(Some(&current), "GET", &path, None).await).await;
    let row = find(&body, &added.id.to_string());
    let joined = row["joined_at"].as_str().unwrap().to_owned();
    let observed = row["repositories"]["observed_at"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(row["repositories"]["visible"], "3");
    assert_eq!(row["repositories"]["private"], "2");
    assert_eq!(row["repositories"]["allowed_private"], "1");
    assert_eq!(row["added_by"]["id"], "7");
    assert_eq!(row["added_by"]["login"], "person-7");
    let removed_at = "2026-11-06T12:00:00Z".parse().unwrap();
    fixture
        .database
        .store
        .record_app_removal(fixture.workspace, added.id, removed_at)
        .unwrap();
    let body = json(fixture.request(Some(&current), "GET", &path, None).await).await;
    let row = find(&body, &added.id.to_string());
    assert_eq!(row["removed_at"], "2026-11-06T12:00:00Z");
    assert_eq!(row["delete_after"], "2026-12-06T12:00:00Z");
    assert_eq!(row["repositories"]["observed_at"], observed);
    fixture.database.store.install_organization(&added).unwrap();
    let body = json(fixture.request(Some(&current), "GET", &path, None).await).await;
    let row = find(&body, &added.id.to_string());
    assert_eq!(row["joined_at"], joined);
    assert_eq!(row["removed_at"], Value::Null);
    assert_eq!(row["delete_after"], Value::Null);
    assert_eq!(row["added_by"]["id"], "7");
    server
        .control
        .person
        .store(8, std::sync::atomic::Ordering::SeqCst);
    let replacement_admin = fixture.login(8, Some(Role::Admin)).await;
    assert_eq!(
        fixture
            .request(
                Some(&replacement_admin),
                "DELETE",
                &fixture.path("/members/7"),
                None
            )
            .await
            .status(),
        StatusCode::OK
    );
    let body = json(
        fixture
            .request(Some(&replacement_admin), "GET", &path, None)
            .await,
    )
    .await;
    assert_eq!(find(&body, &added.id.to_string())["added_by"]["id"], "7");
    assert_eq!(
        find(&body, &added.id.to_string())["added_by"]["login"],
        Value::Null
    );
    let mut connection = PgConnection::establish(&fixture.database.url).unwrap();
    diesel::sql_query("UPDATE organizations SET joined_at=NULL WHERE id=$1")
        .bind::<diesel::sql_types::Uuid, _>(added.id)
        .execute(&mut connection)
        .unwrap();
    fixture.database.store.install_organization(&added).unwrap();
    let body = json(
        fixture
            .request(Some(&replacement_admin), "GET", &path, None)
            .await,
    )
    .await;
    assert_eq!(find(&body, &added.id.to_string())["joined_at"], Value::Null);
    fixture
        .database
        .store
        .begin_organization_takeout(fixture.workspace, added.id)
        .unwrap();
    let body = json(
        fixture
            .request(Some(&replacement_admin), "GET", &path, None)
            .await,
    )
    .await;
    let row = find(&body, &added.id.to_string());
    assert_eq!(row["removed_at"], Value::Null);
    assert_eq!(row["delete_after"], Value::Null);
}

fn repository(id: i64, private: bool) -> Repository {
    Repository {
        id,
        full_name: format!("not-registered/repo-{id}"),
        private,
        default_branch: "main".into(),
    }
}

fn find<'a>(body: &'a Value, id: &str) -> &'a Value {
    body.as_array()
        .unwrap()
        .iter()
        .find(|organization| organization["id"] == id)
        .unwrap()
}
