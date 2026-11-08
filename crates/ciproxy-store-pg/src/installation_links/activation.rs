//! Independent admission holds and current execution authority.

use ciproxy_store_interface::error::{Error, Result};
use ciproxy_store_interface::installation_links::OrganizationActivation;
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Bool, Nullable};
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::{QueryResultExt, require_record};

use super::transaction;

#[derive(QueryableByName)]
struct Row {
    #[diesel(sql_type = Bool)]
    admitted: bool,
    #[diesel(sql_type = Bool)]
    billing_hold: bool,
    #[diesel(sql_type = Bool)]
    admin_approval_hold: bool,
    #[diesel(sql_type = Nullable<BigInt>)]
    added_by: Option<i64>,
}

#[derive(QueryableByName)]
struct Claim {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = BigInt)]
    requester_id: i64,
    #[diesel(sql_type = Bool)]
    approved: bool,
    #[diesel(sql_type = Bool)]
    direct: bool,
}

pub(super) fn read(
    store: &PgStore,
    workspace: Uuid,
    organization: Uuid,
) -> Result<OrganizationActivation> {
    diesel::sql_query("SELECT admitted,billing_hold,admin_approval_hold,added_by FROM organization_activation WHERE workspace_id=$1 AND organization_id=$2")
        .bind::<diesel::sql_types::Uuid,_>(workspace).bind::<diesel::sql_types::Uuid,_>(organization)
        .get_result::<Row>(&mut *store.connection()?).optional().store()?
        .map(|row| OrganizationActivation {admitted:row.admitted,billing_hold:row.billing_hold,admin_approval_hold:row.admin_approval_hold,added_by:row.added_by}).ok_or(Error::NotFound)
}

#[derive(QueryableByName)]
struct Allowed {
    #[diesel(sql_type = Bool)]
    allowed: bool,
}

pub(super) fn runnable(store: &PgStore, workspace: Uuid, organization: Uuid) -> Result<bool> {
    Ok(diesel::sql_query("SELECT EXISTS(SELECT 1 FROM organizations o JOIN organization_activation a ON a.organization_id=o.id AND a.workspace_id=o.workspace_id WHERE o.workspace_id=$1 AND o.id=$2 AND o.app_state='active' AND a.admitted) AS allowed")
        .bind::<diesel::sql_types::Uuid,_>(workspace).bind::<diesel::sql_types::Uuid,_>(organization)
        .get_result::<Allowed>(&mut *store.connection()?).store()?.allowed)
}

pub(super) fn billing(store: &PgStore, workspace: Uuid) -> Result<()> {
    transaction::run(&mut *store.connection()?, |connection| {
        diesel::sql_query("SELECT id FROM workspaces WHERE id=$1 FOR UPDATE")
            .bind::<diesel::sql_types::Uuid, _>(workspace)
            .execute(connection)
            .store()?;
        refresh_billing(connection, workspace).store()?;
        Ok(())
    })
}

/// Recheck linked count only while the caller owns the workspace row lock.
pub(crate) fn refresh_billing(connection: &mut PgConnection, workspace: Uuid) -> QueryResult<()> {
    diesel::sql_query("UPDATE organization_activation a SET billing_hold=COALESCE((SELECT state_json::jsonb->>'tier'='free' FROM billing_accounts WHERE workspace_id=$1),TRUE) AND (SELECT count(*) FROM organizations WHERE workspace_id=$1 AND app_state<>'removed')>1 WHERE a.workspace_id=$1 AND NOT admitted")
        .bind::<diesel::sql_types::Uuid,_>(workspace).execute(connection)?;
    Ok(())
}

pub(super) fn approve(store: &PgStore, workspace: Uuid, organization: Uuid) -> Result<()> {
    require_record(diesel::sql_query("UPDATE organization_activation SET admin_approval_hold=FALSE WHERE workspace_id=$1 AND organization_id=$2")
        .bind::<diesel::sql_types::Uuid,_>(workspace).bind::<diesel::sql_types::Uuid,_>(organization)
        .execute(&mut *store.connection()?).store()?)
}

#[derive(QueryableByName)]
struct Workspace {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    workspace_id: Uuid,
}

pub(super) fn pending(store: &PgStore) -> Result<Vec<Uuid>> {
    Ok(diesel::sql_query("SELECT DISTINCT workspace_id FROM organization_activation WHERE NOT admitted ORDER BY workspace_id")
        .load::<Workspace>(&mut *store.connection()?).store()?.into_iter().map(|row|row.workspace_id).collect())
}

/// Called only inside the registration transaction after its workspace lock.
pub(crate) fn register(
    connection: &mut PgConnection,
    workspace: Uuid,
    organization: Uuid,
    requester: i64,
    approved: bool,
) -> QueryResult<()> {
    diesel::sql_query("INSERT INTO organization_activation (organization_id,workspace_id,admitted,billing_hold,admin_approval_hold,added_by) SELECT $2,$1,FALSE,COALESCE((SELECT state_json::jsonb->>'tier'='free' FROM billing_accounts WHERE workspace_id=$1),TRUE) AND (SELECT count(*) FROM organizations WHERE workspace_id=$1 AND app_state<>'removed')>1,NOT ($4 AND EXISTS (SELECT 1 FROM dashboard_roles r JOIN dashboard_users u ON u.github_id=r.github_user_id JOIN dashboard_memberships m ON m.user_id=u.id JOIN organizations o ON o.github_id=m.github_id AND o.workspace_id=r.workspace_id WHERE r.workspace_id=$1 AND r.github_user_id=$3 AND r.role='admin' AND o.app_state<>'removed' AND o.id<>$2)),$3 ON CONFLICT(organization_id) DO NOTHING")
        .bind::<diesel::sql_types::Uuid,_>(workspace).bind::<diesel::sql_types::Uuid,_>(organization)
        .bind::<BigInt,_>(requester).bind::<Bool,_>(approved).execute(connection)?;
    Ok(())
}

pub(crate) fn permitted(connection: &mut PgConnection, organization: Uuid) -> QueryResult<bool> {
    Ok(snapshot(connection, organization)?.permits_activation())
}

pub(crate) fn admit(connection: &mut PgConnection, organization: Uuid) -> QueryResult<()> {
    diesel::sql_query("UPDATE organization_activation SET admitted=TRUE WHERE organization_id=$1 AND NOT billing_hold AND NOT admin_approval_hold")
        .bind::<diesel::sql_types::Uuid,_>(organization).execute(connection)?;
    Ok(())
}

/// Bind only verified exact choices; a normal install creates no local holds.
pub(crate) fn link(
    connection: &mut PgConnection,
    workspace: Uuid,
    organization: Uuid,
    installation: i64,
) -> QueryResult<bool> {
    let claim=diesel::sql_query("SELECT id,requester_id,COALESCE(admin_approved,FALSE) AS approved,direct FROM installation_claims WHERE installation_id=$1 AND event_seen AND (direct OR (workspace_id=$2 AND user_verified))")
        .bind::<BigInt,_>(installation).bind::<diesel::sql_types::Uuid,_>(workspace)
        .get_result::<Claim>(connection).optional()?;
    if let Some(claim) = claim {
        if claim.direct {
            initialize_admitted(connection, workspace, organization)?;
        } else {
            register(
                connection,
                workspace,
                organization,
                claim.requester_id,
                claim.approved,
            )?;
        }
        diesel::sql_query(
            "UPDATE installation_claims SET organization_id=$2,workspace_id=$3 WHERE id=$1",
        )
        .bind::<diesel::sql_types::Uuid, _>(claim.id)
        .bind::<diesel::sql_types::Uuid, _>(organization)
        .bind::<diesel::sql_types::Uuid, _>(workspace)
        .execute(connection)?;
        return Ok(true);
    }
    Ok(false)
}

pub(crate) fn snapshot(
    connection: &mut PgConnection,
    organization: Uuid,
) -> QueryResult<OrganizationActivation> {
    diesel::sql_query("SELECT admitted,billing_hold,admin_approval_hold,added_by FROM organization_activation WHERE organization_id=$1")
        .bind::<diesel::sql_types::Uuid,_>(organization).get_result::<Row>(connection).optional()?
        .map(|row|OrganizationActivation{admitted:row.admitted,billing_hold:row.billing_hold,admin_approval_hold:row.admin_approval_hold,added_by:row.added_by}).ok_or(diesel::result::Error::NotFound)
}

/// Explicit trusted direct insertion, preserving every retained local hold.
pub(crate) fn initialize_admitted(
    connection: &mut PgConnection,
    workspace: Uuid,
    organization: Uuid,
) -> QueryResult<()> {
    diesel::sql_query("INSERT INTO organization_activation (organization_id,workspace_id,admitted,billing_hold,admin_approval_hold) VALUES ($1,$2,TRUE,FALSE,FALSE) ON CONFLICT(organization_id) DO NOTHING")
        .bind::<diesel::sql_types::Uuid,_>(organization).bind::<diesel::sql_types::Uuid,_>(workspace).execute(connection)?;
    Ok(())
}
