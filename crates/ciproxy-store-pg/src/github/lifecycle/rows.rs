//! Scoped organization snapshots and prepared-resource writes within a transaction.

use ciproxy_schema::organizations::organizations;
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::github_lifecycle::{GithubLifecycleTarget, LifecycleEvent};
use ciproxy_store_interface::organizations::{AppState, Organization};
use diesel::prelude::*;
use diesel::result::Error;
use uuid::Uuid;

use crate::github::{repositories, scale_sets};
use crate::installation_links::activation;
use crate::organization_rows::OrganizationRow;

pub(super) fn lock_workspace(connection: &mut PgConnection, workspace_id: Uuid) -> QueryResult<()> {
    workspaces::table
        .find(workspace_id)
        .select(workspaces::id)
        .for_update()
        .first::<Uuid>(connection)
        .optional()?;
    Ok(())
}

pub(super) fn optional(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    organization_id: Uuid,
) -> QueryResult<Option<Organization>> {
    organizations::table
        .find(organization_id)
        .filter(organizations::workspace_id.eq(workspace_id))
        .select(OrganizationRow::as_select())
        .for_update()
        .first::<OrganizationRow>(connection)
        .optional()?
        .map(|row| match row.into_record() {
            Ok(record) => Ok(record),
            Err(source) => Err(Error::DeserializationError(Box::new(source))),
        })
        .transpose()
}

pub(super) fn load(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    organization_id: Uuid,
) -> QueryResult<Organization> {
    optional(connection, workspace_id, organization_id)?.ok_or(Error::NotFound)
}

pub(super) fn resources(
    connection: &mut PgConnection,
    organization: &Organization,
    target: &GithubLifecycleTarget,
) -> QueryResult<()> {
    activation::refresh_billing(connection, organization.workspace_id)?;
    if !activation::permitted(connection, organization.id)? {
        return Ok(());
    }
    if let GithubLifecycleTarget::Active(prepared) = target {
        repositories::replace_on(
            connection,
            organization.workspace_id,
            organization.id,
            &prepared.repositories,
        )?;
        for scale_set in &prepared.scale_sets {
            scale_sets::save_on(connection, organization.workspace_id, scale_set)?;
        }
        activation::admit(connection, organization.id)?;
    }
    Ok(())
}

pub(super) fn state(target: &GithubLifecycleTarget) -> AppState {
    match target {
        GithubLifecycleTarget::Active(_) => AppState::Active,
        GithubLifecycleTarget::Suspended => AppState::Suspended,
        GithubLifecycleTarget::Inactive => AppState::Active,
    }
}

pub(super) fn event(
    connection: &mut PgConnection,
    organization: Organization,
) -> QueryResult<LifecycleEvent> {
    let local = activation::snapshot(connection, organization.id)?;
    if organization.app_state == AppState::Active && !local.admitted {
        return Ok(LifecycleEvent::Inactive {
            organization,
            billing_hold: local.billing_hold,
            admin_approval_hold: local.admin_approval_hold,
        });
    }
    if organization.app_state == AppState::Suspended {
        Ok(LifecycleEvent::Suspended(organization))
    } else {
        Ok(LifecycleEvent::Active(organization))
    }
}
