//! Browser continuation lifetime and session issuance use the real database.

use chrono::{DateTime, Duration, Utc};
use ciproxy_schema::dashboard::dashboard_continuations;
use ciproxy_store_interface::error::Error;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use uuid::Uuid;

use super::support::Database;
use super::support::installation_results::{SignIn, pending_result, sign_in};

#[test]
fn continuation_survives_person_pruning_and_expires_at_the_fixed_boundary() {
    let mut database = Database::new();
    let SignIn {
        user,
        memberships,
        session,
    } = sign_in(&database);
    let claim = pending_result(&database, 1066).id;
    let results = database.store.pending_installation_results(7).unwrap();
    assert!(
        database
            .store
            .establish_dashboard_session(&user, &memberships, &session, &results, None)
            .unwrap()
    );
    database.store.save_dashboard_user(&user, &[]).unwrap();
    assert!(database.store.dashboard_user(user.id).unwrap().is_none());
    assert!(
        database
            .store
            .dashboard_session(&session.digest, user.checked_at)
            .unwrap()
            .is_none()
    );
    let just_before = session.expires_at - Duration::seconds(1);
    database.store.expire_dashboard_auth(just_before).unwrap();
    assert_eq!(
        database
            .store
            .dashboard_continuation(session.id, just_before)
            .unwrap(),
        vec![claim]
    );
    assert_eq!(
        database
            .store
            .dashboard_continuation(session.id, session.expires_at)
            .unwrap(),
        Vec::<Uuid>::new()
    );
    assert_eq!(
        dashboard_continuations::table
            .find(session.id)
            .select(dashboard_continuations::expires_at)
            .first::<DateTime<Utc>>(&mut database.connection)
            .unwrap(),
        session.expires_at
    );
    database
        .store
        .expire_dashboard_auth(session.expires_at)
        .unwrap();
    assert_eq!(
        dashboard_continuations::table
            .count()
            .get_result::<i64>(&mut database.connection)
            .unwrap(),
        0
    );
}

#[test]
fn failed_continuation_write_rolls_back_identity_and_session_rotation() {
    let mut database = Database::new();
    let SignIn {
        mut user,
        memberships,
        mut session,
    } = sign_in(&database);
    let claim = pending_result(&database, 1066).id;
    let results = database.store.pending_installation_results(7).unwrap();
    assert!(
        database
            .store
            .establish_dashboard_session(&user, &memberships, &session, &results, None)
            .unwrap()
    );
    assert_eq!(
        database.store.pending_installation_results(7).unwrap(),
        vec![]
    );
    let pending = pending_result(&database, 2066).id;
    let results = database.store.pending_installation_results(7).unwrap();
    assert_eq!(
        results
            .iter()
            .map(|result| result.claim_id)
            .collect::<Vec<_>>(),
        vec![pending]
    );
    database
        .connection
        .batch_execute(
            "CREATE FUNCTION fail_continuation() RETURNS trigger AS $$
         BEGIN RAISE EXCEPTION 'controlled continuation write failure'; END
         $$ LANGUAGE plpgsql;
         CREATE TRIGGER fail_continuation BEFORE INSERT OR UPDATE ON dashboard_continuations
         FOR EACH ROW EXECUTE FUNCTION fail_continuation();",
        )
        .unwrap();
    user.credentials = vec![21];
    user.checked_at += Duration::minutes(1);
    session.digest = vec![22];
    session.csrf_digest = vec![23];
    let failure =
        database
            .store
            .establish_dashboard_session(&user, &memberships, &session, &results, None);
    assert!(matches!(failure, Err(Error::Internal(_))));
    let saved_user = database.store.dashboard_user(user.id).unwrap().unwrap();
    assert_eq!(saved_user.credentials, vec![11]);
    assert_eq!(
        saved_user.checked_at.to_rfc3339(),
        "2026-11-07T12:00:00+00:00"
    );
    let saved_session = database
        .store
        .dashboard_session(&[12], user.checked_at)
        .unwrap()
        .unwrap();
    assert_eq!(saved_session.id, session.id);
    assert_eq!(saved_session.csrf_digest, vec![13]);
    assert!(
        database
            .store
            .dashboard_session(&[22], user.checked_at)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        database
            .store
            .dashboard_continuation(session.id, user.checked_at)
            .unwrap(),
        vec![claim]
    );
    assert_eq!(
        database.store.pending_installation_results(7).unwrap(),
        results
    );
    database
        .connection
        .batch_execute("DROP TRIGGER fail_continuation ON dashboard_continuations")
        .unwrap();
    assert!(
        database
            .store
            .establish_dashboard_session(&user, &memberships, &session, &results, None)
            .unwrap()
    );
    assert_eq!(
        database.store.pending_installation_results(7).unwrap(),
        vec![]
    );
    assert_eq!(
        database
            .store
            .dashboard_continuation(session.id, user.checked_at)
            .unwrap(),
        vec![pending]
    );
}
