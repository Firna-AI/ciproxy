//! Private rows for opaque dashboard credentials and member metadata.

use chrono::{DateTime, Utc};
use ciproxy_schema::dashboard::{dashboard_login_states, dashboard_sessions, dashboard_users};
use ciproxy_store_interface::dashboard::records::{LoginState, Member, Session, User};
use ciproxy_store_interface::error::{Result, ResultExt};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Nullable, Text, Timestamptz};
use uuid::Uuid;

#[derive(Queryable, Selectable)]
#[diesel(table_name = dashboard_users)]
pub(super) struct UserRow {
    id: Uuid,
    github_id: i64,
    login: String,
    avatar_url: String,
    credentials: Vec<u8>,
    checked_at: DateTime<Utc>,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id,
            github_id: row.github_id,
            login: row.login,
            avatar_url: row.avatar_url,
            credentials: row.credentials,
            checked_at: row.checked_at,
        }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = dashboard_sessions)]
pub(super) struct SessionRow {
    id: Uuid,
    user_id: Uuid,
    digest: Vec<u8>,
    csrf_digest: Vec<u8>,
    expires_at: DateTime<Utc>,
}

impl From<SessionRow> for Session {
    fn from(row: SessionRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            digest: row.digest,
            csrf_digest: row.csrf_digest,
            expires_at: row.expires_at,
        }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = dashboard_login_states)]
pub(super) struct LoginRow {
    id: Uuid,
    digest: Vec<u8>,
    verifier: Vec<u8>,
    expires_at: DateTime<Utc>,
}

impl From<LoginRow> for LoginState {
    fn from(row: LoginRow) -> Self {
        Self {
            id: row.id,
            digest: row.digest,
            verifier: row.verifier,
            expires_at: row.expires_at,
        }
    }
}

#[derive(QueryableByName)]
pub(super) struct MemberRow {
    #[diesel(sql_type = BigInt)]
    pub(super) github_user_id: i64,
    #[diesel(sql_type = Nullable<Text>)]
    pub(super) login: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub(super) role: Option<String>,
    #[diesel(sql_type = Timestamptz)]
    pub(super) requested_at: DateTime<Utc>,
}

impl MemberRow {
    pub(super) fn into_record(self) -> Result<Member> {
        Ok(Member {
            github_user_id: self.github_user_id,
            login: self.login,
            role: self.role.map(|role| role.parse().internal()).transpose()?,
            requested_at: self.requested_at,
        })
    }
}
