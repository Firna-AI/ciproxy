//! One-use OAuth attempts and fixed-duration opaque browser sessions.

use chrono::{DateTime, Utc};
use ciproxy_schema::dashboard::{dashboard_login_states, dashboard_sessions};
use ciproxy_store_interface::dashboard::records::{LoginState, Membership, Session, User};
use ciproxy_store_interface::error::Result;
use diesel::prelude::*;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;

use super::rows::{LoginRow, SessionRow};

pub(super) fn save_state(store: &PgStore, state: &LoginState) -> Result<()> {
    diesel::insert_into(dashboard_login_states::table)
        .values((
            dashboard_login_states::id.eq(state.id),
            dashboard_login_states::digest.eq(&state.digest),
            dashboard_login_states::verifier.eq(&state.verifier),
            dashboard_login_states::expires_at.eq(state.expires_at),
        ))
        .execute(&mut *store.connection()?)
        .store()?;
    Ok(())
}

pub(super) fn consume_state(
    store: &PgStore,
    digest: &[u8],
    at: DateTime<Utc>,
) -> Result<Option<LoginState>> {
    let row = diesel::delete(
        dashboard_login_states::table.filter(dashboard_login_states::digest.eq(digest)),
    )
    .returning(LoginRow::as_returning())
    .get_result::<LoginRow>(&mut *store.connection()?)
    .optional()
    .store()?;
    Ok(row
        .map(LoginState::from)
        .filter(|state| at < state.expires_at))
}

fn save_on(connection: &mut PgConnection, session: &Session) -> QueryResult<()> {
    diesel::insert_into(dashboard_sessions::table)
        .values((
            dashboard_sessions::id.eq(session.id),
            dashboard_sessions::user_id.eq(session.user_id),
            dashboard_sessions::digest.eq(&session.digest),
            dashboard_sessions::csrf_digest.eq(&session.csrf_digest),
            dashboard_sessions::expires_at.eq(session.expires_at),
        ))
        .on_conflict(dashboard_sessions::id)
        .do_update()
        .set((
            dashboard_sessions::user_id.eq(session.user_id),
            dashboard_sessions::digest.eq(&session.digest),
            dashboard_sessions::csrf_digest.eq(&session.csrf_digest),
            dashboard_sessions::expires_at.eq(session.expires_at),
        ))
        .execute(connection)?;
    Ok(())
}

pub(super) fn establish(
    store: &PgStore,
    user: &User,
    memberships: &[Membership],
    session: &Session,
) -> Result<bool> {
    store
        .connection()?
        .transaction(|connection| {
            super::identity::save_on(connection, user, memberships)?;
            super::identity::requests_on(connection, user.id, user.checked_at)?;
            super::identity::prune_on(connection)?;
            let exists = ciproxy_schema::dashboard::dashboard_users::table
                .find(user.id)
                .count()
                .get_result::<i64>(connection)?;
            if exists == 0 {
                return QueryResult::Ok(false);
            }
            save_on(connection, session)?;
            Ok(true)
        })
        .store()
}

pub(super) fn read(store: &PgStore, digest: &[u8], at: DateTime<Utc>) -> Result<Option<Session>> {
    Ok(dashboard_sessions::table
        .filter(dashboard_sessions::digest.eq(digest))
        .filter(dashboard_sessions::expires_at.gt(at))
        .select(SessionRow::as_select())
        .first(&mut *store.connection()?)
        .optional()
        .store()?
        .map(Session::from))
}

pub(super) fn delete(store: &PgStore, id: Uuid) -> Result<()> {
    diesel::delete(dashboard_sessions::table.find(id))
        .execute(&mut *store.connection()?)
        .store()?;
    Ok(())
}

pub(super) fn expire(store: &PgStore, at: DateTime<Utc>) -> Result<()> {
    store
        .connection()?
        .transaction(|connection| {
            diesel::delete(dashboard_sessions::table.filter(dashboard_sessions::expires_at.le(at)))
                .execute(connection)?;
            diesel::delete(
                dashboard_login_states::table.filter(dashboard_login_states::expires_at.le(at)),
            )
            .execute(connection)?;
            super::identity::prune_on(connection)
        })
        .store()
}
