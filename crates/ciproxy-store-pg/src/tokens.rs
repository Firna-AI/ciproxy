//! Workspace token hashes, scoped authentication, and revocation queries.

use chrono::{DateTime, Duration, Utc};
use ciproxy_schema::runners::runners;
use ciproxy_schema::tokens::tokens;
use ciproxy_store_interface::error::{Error, Result};
use ciproxy_store_interface::secrets::TokenHash;
use ciproxy_store_interface::tokens::{Token, TokenStore, TokenSubject};
use diesel::prelude::*;
use diesel::sql_types::{Binary, Timestamptz};
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::{InternalCause, QueryResultExt, require_record};

#[derive(QueryableByName)]
#[diesel(table_name = tokens)]
struct TokenRow {
    id: Uuid,
    workspace_id: Uuid,
    purpose: String,
    organization_id: Option<Uuid>,
    machine_id: Option<Uuid>,
    runner_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
    revoked_at: Option<DateTime<Utc>>,
}

impl TokenRow {
    fn into_record(self) -> Result<Token> {
        let subject = match (
            self.purpose.as_str(),
            self.organization_id,
            self.machine_id,
            self.runner_id,
        ) {
            ("install", None, None, None) => TokenSubject::Install,
            ("machine", None, Some(id), None) => TokenSubject::Machine(id),
            ("sandbox", Some(organization_id), None, Some(runner_id)) => TokenSubject::Sandbox {
                organization_id,
                runner_id,
            },
            _ => return Err(Error::internal(InternalCause::InvalidRow)),
        };
        Ok(Token {
            id: self.id,
            workspace_id: self.workspace_id,
            subject,
            created_at: self.created_at,
            expires_at: self.expires_at,
            revoked_at: self.revoked_at,
        })
    }
}

pub(crate) fn insert_token(
    connection: &mut PgConnection,
    token: &Token,
    hash: &TokenHash,
) -> QueryResult<()> {
    let (purpose, organization_id, machine_id, runner_id) = match token.subject {
        TokenSubject::Install => ("install", None, None, None),
        TokenSubject::Machine(id) => ("machine", None, Some(id), None),
        TokenSubject::Sandbox {
            organization_id,
            runner_id,
        } => ("sandbox", Some(organization_id), None, Some(runner_id)),
    };
    diesel::insert_into(tokens::table)
        .values((
            tokens::id.eq(token.id),
            tokens::workspace_id.eq(token.workspace_id),
            tokens::token_hash.eq(hash.as_bytes()),
            tokens::purpose.eq(purpose),
            tokens::organization_id.eq(organization_id),
            tokens::machine_id.eq(machine_id),
            tokens::runner_id.eq(runner_id),
            tokens::created_at.eq(token.created_at),
            tokens::expires_at.eq(token.expires_at),
            tokens::revoked_at.eq(token.revoked_at),
        ))
        .execute(connection)?;
    Ok(())
}

impl TokenStore for PgStore {
    fn create_install_token(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        hash: &TokenHash,
        at: DateTime<Utc>,
    ) -> Result<()> {
        insert_token(
            &mut *self.connection()?,
            &Token {
                id,
                workspace_id,
                subject: TokenSubject::Install,
                created_at: at,
                expires_at: Some(at + Duration::hours(1)),
                revoked_at: None,
            },
            hash,
        )
        .store()
    }

    fn create_scoped_token(&self, token: &Token, hash: &TokenHash) -> Result<()> {
        if token.subject == TokenSubject::Install {
            return Err(Error::InvalidValue);
        }
        let mut connection = self.connection()?;
        if let TokenSubject::Sandbox {
            organization_id,
            runner_id,
        } = token.subject
        {
            let sandbox = runners::table
                .filter(runners::workspace_id.eq(token.workspace_id))
                .filter(runners::organization_id.eq(organization_id))
                .filter(runners::sandbox_id.is_not_null())
                .find(runner_id)
                .select(runners::id)
                .first::<Uuid>(&mut *connection)
                .optional()
                .store()?;
            if sandbox.is_none() {
                return Err(Error::InvalidReference);
            }
        }
        insert_token(&mut connection, token, hash).store()
    }

    fn authenticate_token(&self, hash: &TokenHash, at: DateTime<Utc>) -> Result<Option<Token>> {
        let row = diesel::sql_query(
            "SELECT credential.id, credential.workspace_id, credential.purpose,
                 credential.organization_id, credential.machine_id, credential.runner_id,
                 credential.created_at, credential.expires_at, credential.revoked_at
             FROM tokens AS credential
             WHERE credential.token_hash = $1 AND credential.revoked_at IS NULL
             AND (credential.expires_at IS NULL OR credential.expires_at > $2)
             AND (credential.purpose <> 'sandbox' OR EXISTS (
                 SELECT 1 FROM runners
                 WHERE runners.workspace_id = credential.workspace_id
                 AND runners.organization_id = credential.organization_id
                 AND runners.id = credential.runner_id AND runners.sandbox_id IS NOT NULL
                 AND runners.state <> 'stopped'
             ))
             AND (credential.purpose <> 'machine' OR EXISTS (
                 SELECT 1 FROM machines
                 WHERE machines.workspace_id = credential.workspace_id
                 AND machines.id = credential.machine_id AND machines.removed_at IS NULL
             ))",
        )
        .bind::<Binary, _>(hash.as_bytes())
        .bind::<Timestamptz, _>(at)
        .get_result::<TokenRow>(&mut *self.connection()?)
        .optional()
        .store()?;
        row.map(TokenRow::into_record).transpose()
    }

    fn revoke_token(&self, workspace_id: Uuid, id: Uuid, at: DateTime<Utc>) -> Result<()> {
        let count = diesel::update(
            tokens::table
                .filter(tokens::workspace_id.eq(workspace_id))
                .find(id),
        )
        .set(tokens::revoked_at.eq(at))
        .execute(&mut *self.connection()?)
        .store()?;
        require_record(count)
    }
}
