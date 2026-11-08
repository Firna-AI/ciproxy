//! Controlled overlapping policy operations with waiting PostgreSQL leases.

use std::time::Duration;

use ciproxy_github::error::Error;
use ciproxy_store_interface::organizations::RepositoryAccess;
use diesel::prelude::*;
use diesel::sql_types::Bool;
use tokio::time::timeout;

use super::app_fixture::Fixture;
use super::database::Database;
use super::support::{id, setup};

#[derive(QueryableByName)]
struct Waiting {
    #[diesel(sql_type = Bool)]
    waiting: bool,
}

async fn waiting_lease(database_url: &str) {
    let mut connection = PgConnection::establish(database_url).unwrap();
    loop {
        let row = diesel::sql_query("SELECT EXISTS (SELECT 1 FROM pg_locks WHERE locktype = 'advisory' AND objsubid = 2 AND classid = 0::oid AND objid = 17::oid AND NOT granted AND database = (SELECT oid FROM pg_database WHERE datname = current_database())) AS waiting")
            .get_result::<Waiting>(&mut connection).unwrap();
        if row.waiting {
            return;
        }
        tokio::task::yield_now().await;
    }
}

#[tokio::test]
async fn organization_lease_orders_access_restrictions_and_overlapping_repository_snapshots() {
    for restrict in [true, false] {
        let database = Database::new();
        let target = setup(&*database.store);
        let fixture = Fixture::new();
        let first_app = fixture.app(database.store.clone());
        let second_app = fixture.app(database.store.clone());
        let (entered, release) = fixture.remote.hold_next_read();
        let first = tokio::spawn(async move { first_app.refresh_repositories(19).await });
        entered.await.unwrap();
        if !restrict {
            fixture.remote.repositories.lock().unwrap()[0].default_branch =
                "new-default".to_owned();
        }
        let mut second = tokio::spawn(async move {
            if restrict {
                second_app
                    .set_repository_access(19, RepositoryAccess::Selected(vec![23]))
                    .await
            } else {
                second_app.refresh_repositories(19).await
            }
        });
        let mut completed = None;
        let ordered = timeout(Duration::from_secs(10), async {
            tokio::select! {
                _ = waiting_lease(&database.url) => true,
                result = &mut second => { completed = Some(result); false },
            }
        })
        .await;
        let store = database.store.clone();
        let ordinary = timeout(
            Duration::from_secs(10),
            tokio::task::spawn_blocking(move || store.workspace(target.workspace_id)),
        )
        .await;
        let ordinary_ready = matches!(ordinary, Ok(Ok(Ok(Some(_)))));
        if !ordinary_ready || ordered.is_err() {
            first.abort();
        }
        let _ = release.send(());
        let first_result = first.await;
        let second_result = if let Some(result) = completed {
            result
        } else {
            second.await
        };
        assert!(
            matches!(ordered, Ok(true)),
            "restrict={restrict}: second mutation completed before the first lease ended"
        );
        assert!(
            ordinary_ready,
            "restrict={restrict}: advisory wait blocked ordinary queries"
        );
        assert!(
            matches!(first_result, Ok(Ok(()))),
            "restrict={restrict}: first mutation failed"
        );
        assert!(
            matches!(second_result, Ok(Ok(()))),
            "restrict={restrict}: second mutation failed"
        );
        let policies = fixture.remote.policies.lock().unwrap().clone();
        let expected_ids = if restrict { vec![23] } else { vec![23, 29] };
        assert_eq!(
            policies.last().unwrap().ids,
            expected_ids,
            "restrict={restrict}"
        );
        let organization = database
            .store
            .organization(target.workspace_id, target.organization_id)
            .unwrap()
            .unwrap();
        let expected_access = if restrict {
            RepositoryAccess::Selected(vec![23])
        } else {
            RepositoryAccess::AllPrivate
        };
        assert_eq!(
            organization.repositories, expected_access,
            "restrict={restrict}"
        );
        let repositories = database
            .store
            .repositories(target.workspace_id, target.organization_id)
            .unwrap();
        assert_eq!(
            repositories[0].default_branch,
            if restrict {
                "old-default"
            } else {
                "new-default"
            },
            "restrict={restrict}"
        );
    }
}

#[tokio::test]
async fn refused_label_keeps_repository_access_saved_while_validation_waits_for_lease() {
    let database = Database::new();
    let target = setup(&*database.store);
    let fixture = Fixture::new();
    let editor = fixture.app(database.store.clone());
    let validator = fixture.app(database.store.clone());
    let (entered, release) = fixture.remote.hold_next_read();
    let mut edit = tokio::spawn(async move {
        editor
            .set_repository_access(19, RepositoryAccess::Selected(vec![23]))
            .await
    });
    entered.await.unwrap();
    assert_eq!(
        database
            .store
            .organization(target.workspace_id, target.organization_id)
            .unwrap()
            .unwrap()
            .repositories,
        RepositoryAccess::AllPrivate
    );
    let mut label = database
        .store
        .labels(target.workspace_id)
        .unwrap()
        .remove(0);
    label.id = id(88);
    label.name = "refused-label".to_owned();
    let mut validation =
        tokio::spawn(async move { validator.prepare_label(target.workspace_id, label).await });
    let mut completed = None;
    let ordered = timeout(Duration::from_secs(10), async {
        tokio::select! {
            _ = waiting_lease(&database.url) => true,
            result = &mut validation => { completed = Some(result); false },
        }
    })
    .await;
    let waited = matches!(ordered, Ok(true));
    if ordered.is_err() {
        edit.abort();
        validation.abort();
    }
    let _ = release.send(());
    let joined = timeout(Duration::from_secs(10), async {
        let edit_result = (&mut edit).await.unwrap();
        let validation_result = if let Some(result) = completed {
            result
        } else {
            (&mut validation).await
        }
        .unwrap();
        (edit_result, validation_result)
    })
    .await;
    if joined.is_err() {
        edit.abort();
        validation.abort();
    }
    assert!(
        waited,
        "label validation must wait for the real organization lease"
    );
    let (edit_result, validation_result) =
        joined.expect("leased operations must finish after release");
    assert!(
        matches!(edit_result, Ok(())),
        "access edit failed: {edit_result:?}"
    );
    assert!(
        matches!(validation_result, Err(Error::AccessUnavailable(422))),
        "GitHub must refuse the candidate: {validation_result:?}"
    );
    assert_eq!(
        database
            .store
            .organization(target.workspace_id, target.organization_id)
            .unwrap()
            .unwrap()
            .repositories,
        RepositoryAccess::Selected(vec![23])
    );
    let permissions: Vec<_> = fixture
        .remote
        .policies
        .lock()
        .unwrap()
        .iter()
        .map(|policy| policy.ids.clone())
        .collect();
    assert_eq!(
        permissions,
        vec![vec![23], vec![23]],
        "refused label validation must not restore a revoked repository"
    );
}
