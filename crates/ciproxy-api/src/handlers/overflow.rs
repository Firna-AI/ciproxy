//! Workspace routing policy and Admin-only provider connection controls.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::{Extension, Json};
use ciproxy_overflow::api::Overflow;
use ciproxy_store_interface::overflow::provider;
use ciproxy_store_interface::settings::{OverflowWait, Settings};

use crate::access::Principal;
use crate::error::{Error, Problem, Result};
use crate::pages::PageServices;
use crate::state::{ApiState, query};
use crate::wire::access::{Success, WorkspacePath};
use crate::wire::money::Cents;
use crate::wire::overflow::{
    BoatConnection, BoatKeyCheck, BoatWallet, BoatWallets, OverflowPolicy, OverflowProvider,
    OverflowProviderPath, OverflowSettings, ProviderSpendLimit, ProviderSuccess,
};

use super::overflow_views;

/// Check a provider key without saving it. Boat returns its billing wallets.
#[utoipa::path(operation_id="overflow_check_key", post, path="/workspaces/{workspace_id}/overflow/providers/{provider}/key-check", params(OverflowProviderPath), request_body=BoatKeyCheck, responses((status=200,body=BoatWallets),(status=422,description="Provider key refusal or malformed body",body=Problem)), security(("session"=[])))]
pub(crate) async fn check_key(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<OverflowProviderPath>,
    Json(body): Json<BoatKeyCheck>,
) -> Result<Json<BoatWallets>> {
    state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    let provider = path.provider;
    let accounts = provider_result(
        provider,
        service(&state)?.check_key(provider.0, body.api_key).await,
    )?;
    match provider.0 {
        provider::OverflowProvider::Boat => Ok(Json(BoatWallets {
            provider,
            wallets: accounts
                .accounts
                .into_iter()
                .map(|account| BoatWallet {
                    wallet: account.id,
                    name: account.name,
                })
                .collect(),
            selected_wallet: accounts.selected_account,
        })),
    }
}

#[utoipa::path(operation_id="overflow_settings", get, path="/workspaces/{workspace_id}/overflow", params(WorkspacePath), responses((status=200,body=OverflowSettings)), security(("session"=[])))]
pub(crate) async fn settings(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<WorkspacePath>,
    pages: Option<Extension<Arc<PageServices>>>,
) -> Result<Json<OverflowSettings>> {
    let workspace = state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    Ok(Json(
        overflow_views::read(&state, pages.as_deref().map(Arc::as_ref), workspace).await?,
    ))
}

#[utoipa::path(operation_id="overflow_summary", get, path="/workspaces/{workspace_id}/overflow/summary", params(WorkspacePath), responses((status=200,body=OverflowSettings)), security(("session"=[])))]
pub(crate) async fn summary(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<WorkspacePath>,
    pages: Option<Extension<Arc<PageServices>>>,
) -> Result<Json<OverflowSettings>> {
    let workspace = state
        .access
        .require_role(&principal, path.workspace_id, false)
        .await?;
    Ok(Json(
        overflow_views::read(&state, pages.as_deref().map(Arc::as_ref), workspace).await?,
    ))
}

/// Connect a provider key and save its spend limit.
#[utoipa::path(operation_id="overflow_connect", post, path="/workspaces/{workspace_id}/overflow/providers/{provider}/connection", params(OverflowProviderPath), request_body=BoatConnection, responses((status=200,body=ProviderSuccess),(status=422,description="Provider key refusal or malformed body",body=Problem)), security(("session"=[])))]
pub(crate) async fn connect(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<OverflowProviderPath>,
    Json(body): Json<BoatConnection>,
) -> Result<Json<ProviderSuccess>> {
    save_connection(state, &principal, path, body).await
}

/// Replace a provider key while its running jobs finish.
#[utoipa::path(operation_id="overflow_change_key", put, path="/workspaces/{workspace_id}/overflow/providers/{provider}/connection", params(OverflowProviderPath), request_body=BoatConnection, responses((status=200,body=ProviderSuccess),(status=422,description="Provider key refusal or malformed body",body=Problem)), security(("session"=[])))]
pub(crate) async fn change_key(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<OverflowProviderPath>,
    Json(body): Json<BoatConnection>,
) -> Result<Json<ProviderSuccess>> {
    save_connection(state, &principal, path, body).await
}

async fn save_connection(
    state: ApiState,
    principal: &Principal,
    path: OverflowProviderPath,
    body: BoatConnection,
) -> Result<Json<ProviderSuccess>> {
    let workspace = state
        .access
        .require_role(principal, path.workspace_id, true)
        .await?;
    let provider = path.provider;
    let limit = limit_micros(body.spend_limit_cents)?;
    let billing_account = match provider.0 {
        provider::OverflowProvider::Boat => body.wallet,
    };
    provider_result(
        provider,
        service(&state)?
            .connect(
                workspace,
                provider.0,
                state.runtime.identifier(),
                body.api_key,
                billing_account,
                limit,
            )
            .await,
    )?;
    Ok(Json(ProviderSuccess { ok: true, provider }))
}

#[utoipa::path(operation_id="overflow_disconnect", delete, path="/workspaces/{workspace_id}/overflow/providers/{provider}/connection", params(OverflowProviderPath), responses((status=200,body=ProviderSuccess)), security(("session"=[])))]
pub(crate) async fn disconnect(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<OverflowProviderPath>,
) -> Result<Json<ProviderSuccess>> {
    let workspace = state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    let provider = path.provider;
    if let Some(service) = &state.overflow {
        provider_result(provider, service.disconnect(workspace, provider.0).await)?;
    } else {
        let at = state.runtime.now();
        query(state.store, move |store| {
            store.disconnect_overflow_provider(workspace, provider.0, at)?;
            store.delete_unused_overflow_keys(workspace)?;
            Ok(())
        })
        .await?;
    }
    Ok(Json(ProviderSuccess { ok: true, provider }))
}

#[utoipa::path(operation_id="overflow_save_limit", put, path="/workspaces/{workspace_id}/overflow/providers/{provider}/spend-limit", params(OverflowProviderPath), request_body=ProviderSpendLimit, responses((status=200,body=ProviderSuccess)), security(("session"=[])))]
pub(crate) async fn save_limit(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<OverflowProviderPath>,
    Json(body): Json<ProviderSpendLimit>,
) -> Result<Json<ProviderSuccess>> {
    let workspace = state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    let provider = path.provider;
    let limit = limit_micros(body.spend_limit_cents)?;
    query(state.store, move |store| {
        store.save_overflow_provider_spend_limit(workspace, provider.0, Some(limit))
    })
    .await?;
    Ok(Json(ProviderSuccess { ok: true, provider }))
}

#[utoipa::path(operation_id="overflow_save", put, path="/workspaces/{workspace_id}/overflow", params(WorkspacePath), request_body=OverflowPolicy, responses((status=200,body=Success)), security(("session"=[])))]
pub(crate) async fn save(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<WorkspacePath>,
    Json(body): Json<OverflowPolicy>,
) -> Result<Json<Success>> {
    let workspace = state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    let overflow_wait = match body.wait_seconds {
        0 => OverflowWait::AtOnce,
        15 => OverflowWait::Seconds15,
        30 => OverflowWait::Seconds30,
        60 => OverflowWait::Seconds60,
        120 => OverflowWait::Seconds120,
        _ => return Err(Error::InvalidInput),
    };
    let settings = Settings {
        overflow_enabled: body.enabled,
        overflow_wait,
    };
    query(state.store, move |store| {
        store.save_settings(workspace, &settings)
    })
    .await?;
    Ok(Json(Success { ok: true }))
}

fn service(state: &ApiState) -> Result<&dyn Overflow> {
    state.overflow.as_deref().ok_or(Error::Unavailable)
}

fn limit_micros(cents: Cents) -> Result<i64> {
    if cents.0 <= 0 {
        return Err(Error::InvalidInput);
    }
    cents
        .0
        .checked_mul(10_000)
        .and_then(|value| i64::try_from(value).ok())
        .ok_or(Error::InvalidInput)
}

fn provider_result<T>(
    provider: OverflowProvider,
    result: ciproxy_overflow::error::Result<T>,
) -> Result<T> {
    use ciproxy_overflow::error::Error::{ConnectionNotPermitted, InvalidSpendLimit, RejectedKey};

    match result {
        Ok(value) => Ok(value),
        Err(RejectedKey) => Err(Error::ProviderKeyRejected(provider)),
        Err(InvalidSpendLimit) => Err(Error::InvalidInput),
        Err(ConnectionNotPermitted) => Err(Error::Conflict),
        Err(_) => Err(Error::Unavailable),
    }
}
