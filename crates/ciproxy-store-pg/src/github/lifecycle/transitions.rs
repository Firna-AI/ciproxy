//! State, execution revocation, prepared bindings, and notifications commit together.

use chrono::{DateTime, Utc};
use ciproxy_schema::organizations::organizations;
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::github_lifecycle::{
    GithubLifecycleUpdate, GithubRegistration, LifecycleEvent,
};
use ciproxy_store_interface::organizations::{AppState, Organization};
use diesel::prelude::*;
use diesel::result::Error;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;
use crate::github::registration;
use crate::installation_links::activation;
use crate::organization_lifecycle::mark_app_removed;

use super::{queue, rows, state};

pub(super) fn register(store: &PgStore, request: &GithubRegistration) -> Result<Organization> {
    store
        .connection()?
        .transaction(|connection| {
            let mut organization = request.organization.clone();
            organization.app_state = rows::state(&request.target);
            rows::lock_workspace(connection, organization.workspace_id)?;
            let previous = rows::optional(connection, organization.workspace_id, organization.id)?;
            let previous_activation = if previous.is_some() {
                Some(activation::snapshot(connection, organization.id)?)
            } else {
                None
            };
            registration::register(
                connection,
                request.workspace.as_ref(),
                &organization,
                &request.defaults,
                request.initial_admin,
            )?;
            let saved = rows::load(connection, organization.workspace_id, organization.id)?;
            let linked = activation::link(
                connection,
                saved.workspace_id,
                saved.id,
                saved.installation_id,
            )?;
            if !linked {
                if request.workspace.is_none() && previous.is_none() {
                    return Err(Error::NotFound);
                }
                activation::initialize_admitted(connection, saved.workspace_id, saved.id)?;
            }
            rows::resources(connection, &saved, &request.target)?;
            let current_activation = activation::snapshot(connection, saved.id)?;
            if previous.as_ref() != Some(&saved) || previous_activation != Some(current_activation)
            {
                let event = rows::event(connection, saved.clone())?;
                queue::insert(connection, &event)?;
            }
            Ok(saved)
        })
        .store()
}

pub(super) fn update(
    store: &PgStore,
    workspace_id: Uuid,
    organization_id: Uuid,
    update: &GithubLifecycleUpdate,
) -> Result<Organization> {
    store
        .connection()?
        .transaction(|connection| {
            rows::lock_workspace(connection, workspace_id)?;
            let previous = rows::load(connection, workspace_id, organization_id)?;
            let previous_activation = activation::snapshot(connection, organization_id)?;
            let saved = state::save(connection, workspace_id, organization_id, update)?;
            rows::resources(connection, &saved, &update.target)?;
            if saved != previous
                || update.notify
                || activation::snapshot(connection, organization_id)? != previous_activation
            {
                let event = rows::event(connection, saved.clone())?;
                queue::insert(connection, &event)?;
            }
            Ok(saved)
        })
        .store()
}

pub(super) fn remove(
    store: &PgStore,
    workspace_id: Uuid,
    organization_id: Uuid,
    installation_id: i64,
    at: DateTime<Utc>,
) -> Result<Organization> {
    store
        .connection()?
        .transaction(|connection| {
            rows::lock_workspace(connection, workspace_id)?;
            let previous = rows::load(connection, workspace_id, organization_id)?;
            if previous.installation_id != installation_id {
                return Err(Error::NotFound);
            }
            if previous.app_state == AppState::Removed {
                return Ok(previous);
            }
            mark_app_removed(connection, workspace_id, organization_id, at)?;
            let saved = rows::load(connection, workspace_id, organization_id)?;
            let remaining = organizations::table
                .filter(organizations::workspace_id.eq(workspace_id))
                .filter(organizations::app_state.ne("removed"))
                .count()
                .get_result::<i64>(connection)?;
            queue::insert(
                connection,
                &LifecycleEvent::Removed {
                    organization: saved.clone(),
                    last_organization: remaining == 0,
                },
            )?;
            Ok(saved)
        })
        .store()
}
