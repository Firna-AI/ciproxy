//! Browser continuation lifetime and session issuance use the real database.

use chrono::{DateTime, Duration, Utc};
use ciproxy_schema::dashboard::dashboard_continuations;
use ciproxy_store_interface::dashboard::records::{Membership, Session, User};
use ciproxy_store_interface::error::Error;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use uuid::Uuid;

use super::support::Database;

struct SignIn {
    user: User,
    memberships: Vec<Membership>,
    session: Session,
}

fn sign_in(database: &Database) -> SignIn {
    let workspace = database.workspace();
    let organization = database.organization(&workspace);
    let now = "2026-11-07T12:00:00Z".parse().unwrap();
    let user = User {
        id: Uuid::new_v4(),
        github_id: 7,
        login: "person-7".to_owned(),
        avatar_url: "https://github.example/avatar".to_owned(),
        credentials: vec![11],
        checked_at: now,
    };
    let session = Session {
        id: Uuid::new_v4(),
        user_id: user.id,
        digest: vec![12],
        csrf_digest: vec![13],
        expires_at: now + Duration::days(30),
    };
    SignIn {
        user,
        memberships: vec![Membership {
            github_id: organization.github_id,
            owner: false,
        }],
        session,
    }
}

#[test]
fn continuation_survives_person_pruning_and_expires_at_the_fixed_boundary() {
    let mut database = Database::new();
    let SignIn {
        user,
        memberships,
        session,
    } = sign_in(&database);
    let claim = Uuid::new_v4();
    assert!(
        database
            .store
            .establish_dashboard_session(&user, &memberships, &session, &[claim])
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
    let claim = Uuid::new_v4();
    assert!(
        database
            .store
            .establish_dashboard_session(&user, &memberships, &session, &[claim])
            .unwrap()
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
    let failure = database
        .store
        .establish_dashboard_session(&user, &memberships, &session, &[]);
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
}
