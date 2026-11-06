//! Workspace-scoped machine views and real gateway controls.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::{Extension, Json};
use ciproxy_gateway::types::{Delivery, MachineAction};
use ciproxy_protocol::resources::{GIGABYTE, VmSize};
use ciproxy_store_interface::metering::history::MachineCost;

use crate::access::Principal;
use crate::error::{Error, ErrorResultExt, Result, ServiceResultExt};
use crate::state::{ApiState, query};
use crate::wire::access::{Success, WorkspacePath};
use crate::wire::machines::{
    CommandResult, InstallCommand, MachineCommand, MachineCommandPath, MachinePath,
    MachineSettings, MachineView, TokenPath,
};

use super::machine_views;

#[utoipa::path(operation_id = "machines_list", get, path = "/workspaces/{workspace_id}/machines", params(WorkspacePath), responses((status = 200, body = [MachineView])), security(("session" = [])))]
pub(crate) async fn list(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<WorkspacePath>,
) -> Result<Json<Vec<MachineView>>> {
    state
        .access
        .require_role(&principal, path.workspace_id, false)
        .await?;
    Ok(Json(read(&state, path.workspace_id).await?))
}

pub(super) async fn read(state: &ApiState, workspace: uuid::Uuid) -> Result<Vec<MachineView>> {
    let gateway = state.gateway.clone();
    let values = tokio::task::spawn_blocking(move || gateway.machines(workspace))
        .await
        .internal()?
        .api()?;
    let costs = query(state.store.clone(), move |store| {
        store.dashboard_machine_costs(workspace)
    })
    .await?;
    let labels = query(state.store.clone(), move |store| store.labels(workspace)).await?;
    let default_size = labels
        .into_iter()
        .find(|label| label.name == "ciproxy")
        .map(|label| VmSize {
            vcpu: label.size.vcpus.try_into().unwrap_or(0),
            memory_bytes: u64::try_from(label.size.memory_gb)
                .unwrap_or(0)
                .saturating_mul(GIGABYTE),
            disk_bytes: u64::try_from(label.size.disk_gb)
                .unwrap_or(0)
                .saturating_mul(GIGABYTE),
        });
    Ok(values
        .into_iter()
        .map(|value| {
            let capacity = state.capacity.capacity(&value.snapshot);
            let default_size = default_size.and_then(|size| {
                state
                    .capacity
                    .required_root_bytes(&value.snapshot, size.disk_bytes)
                    .map(|disk_bytes| VmSize { disk_bytes, ..size })
            });
            machine_views::machine(value, &costs, state.runtime.now(), default_size, capacity)
        })
        .collect())
}

#[utoipa::path(operation_id = "machines_detail", get, path = "/workspaces/{workspace_id}/machines/{machine_id}", params(MachinePath), responses((status = 200, body = MachineView)), security(("session" = [])))]
pub(crate) async fn detail(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<MachinePath>,
) -> Result<Json<MachineView>> {
    state
        .access
        .require_role(&principal, path.workspace_id, false)
        .await?;
    let id = path.machine_id.to_string();
    Ok(Json(
        read(&state, path.workspace_id)
            .await?
            .into_iter()
            .find(|machine| machine.id == id)
            .ok_or(Error::NotFound)?,
    ))
}

#[utoipa::path(operation_id = "machines_settings", put, path = "/workspaces/{workspace_id}/machines/{machine_id}", params(MachinePath), request_body = MachineSettings, responses((status = 200, body = Success)), security(("session" = [])))]
pub(crate) async fn settings(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<MachinePath>,
    Json(body): Json<MachineSettings>,
) -> Result<Json<Success>> {
    state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    if body.warm_runners < 0 || body.monthly_cost_cents.0 < 0 {
        return Err(Error::InvalidInput);
    }
    let cost = MachineCost {
        id: state.runtime.identifier(),
        machine_id: path.machine_id,
        starts_at: state.runtime.now(),
        ends_at: None,
        monthly_usd: match body.monthly_cost_cents.usd() {
            Ok(value) => value,
            Err(_) => return Err(Error::InvalidInput),
        },
    };
    query(state.store, move |store| {
        store.save_dashboard_machine_settings(path.workspace_id, body.warm_runners, &cost)
    })
    .await?;
    Ok(Json(Success { ok: true }))
}

#[utoipa::path(operation_id = "machines_command", post, path = "/workspaces/{workspace_id}/machines/{machine_id}/actions/{action}", params(MachineCommandPath), responses((status = 200, body = CommandResult)), security(("session" = [])))]
pub(crate) async fn command(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<MachineCommandPath>,
) -> Result<Json<CommandResult>> {
    state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    let exists = query(state.store.clone(), move |store| {
        Ok(store
            .machines(path.workspace_id)?
            .iter()
            .any(|machine| machine.id == path.machine_id))
    })
    .await?;
    if !exists {
        return Err(Error::NotFound);
    }
    let action = match path.action {
        MachineCommand::Drain => MachineAction::Drain,
        MachineCommand::Resume => MachineAction::Resume,
        MachineCommand::Remove => MachineAction::Remove,
        MachineCommand::ClearCache => MachineAction::ClearCache,
    };
    let gateway = state.gateway;
    let id = principal.id();
    let delivered = tokio::task::spawn_blocking(move || {
        gateway.manage(id, path.workspace_id, path.machine_id, action)
    })
    .await
    .internal()?
    .api()?;
    if action == MachineAction::ClearCache && delivered == Delivery::Offline {
        return Err(Error::Unavailable);
    }
    Ok(Json(CommandResult {
        connected: delivered == Delivery::Connected,
    }))
}

#[utoipa::path(operation_id = "machines_issue_token", post, path = "/workspaces/{workspace_id}/install-tokens", params(WorkspacePath), responses((status = 200, body = InstallCommand)), security(("session" = [])))]
pub(crate) async fn issue_token(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<WorkspacePath>,
) -> Result<Json<InstallCommand>> {
    state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    let gateway = state.gateway;
    let id = principal.id();
    let token = tokio::task::spawn_blocking(move || gateway.issue_token(id, path.workspace_id))
        .await
        .internal()?
        .api()?;
    Ok(Json(InstallCommand {
        id: token.id.to_string(),
        command: format!(
            "curl -fsSL {} | sudo sh -s -- install --token {}",
            state.config.download_url,
            token.token.expose_secret()
        ),
        expires_at: token.expires_at,
        requirements: "x64 Linux with KVM, 4 cores, 16 GB of memory, and at least 250 GB of disk."
            .to_owned(),
    }))
}

#[utoipa::path(operation_id = "machines_revoke_token", delete, path = "/workspaces/{workspace_id}/install-tokens/{token_id}", params(TokenPath), responses((status = 200, body = Success)), security(("session" = [])))]
pub(crate) async fn revoke_token(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<TokenPath>,
) -> Result<Json<Success>> {
    state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    let gateway = state.gateway;
    let id = principal.id();
    tokio::task::spawn_blocking(move || gateway.revoke_token(id, path.workspace_id, path.token_id))
        .await
        .internal()?
        .api()?;
    Ok(Json(Success { ok: true }))
}
