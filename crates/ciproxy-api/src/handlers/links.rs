//! Exact original-person installation recovery and current Admin enable commands.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::{Extension, Json};
use ciproxy_store_interface::installation_links::InstallationAttempt;
use reqwest::Url;
use secrecy::ExposeSecret;

use crate::access::authentication::digest;
use crate::access::{InstallContext, Principal};
use crate::error::{Error, ErrorResultExt, Result, ServiceResultExt};
use crate::state::{ApiState, query};
use crate::wire::access::{Success, WorkspacePath};
use crate::wire::links::{ClaimPath, InstallLink, OrganizationLinkView, WorkspaceChoice};
use crate::wire::settings::OrganizationPath;

use super::link_views;

pub(super) async fn install_link(
    state: &ApiState,
    person: i64,
    direct: bool,
) -> Result<InstallLink> {
    let nonce = state.runtime.secret()?;
    let attempt = InstallationAttempt {
        installation_id: None,
        id: state.runtime.identifier(),
        state_digest: digest(nonce.expose_secret()),
        requester_id: person,
        direct,
    };
    let id = attempt.id;
    query(state.store.clone(), move |store| {
        store.begin_installation_attempt(&attempt)
    })
    .await?;
    let mut url = Url::parse(&state.config.github_install_url).internal()?;
    url.set_query(None);
    url.query_pairs_mut()
        .append_pair("state", nonce.expose_secret());
    Ok(InstallLink {
        attempt_id: id,
        url: url.to_string(),
    })
}

async fn continuation_link(state: &ApiState, principal: &Principal) -> Result<InstallLink> {
    let direct = match &principal.install_context {
        Some(InstallContext::Attempt(id)) => {
            let id = *id;
            let attempt = query(state.store.clone(), move |store| {
                store.installation_attempt_by_id(id)
            })
            .await?
            .ok_or(Error::Forbidden)?;
            if attempt.requester_id != principal.github_id() {
                return Err(Error::Forbidden);
            }
            if !attempt.direct
                && let Some(installation) = attempt.installation_id
            {
                let linked = query(state.store.clone(), move |store| {
                    store.claim_for_installation(installation)
                })
                .await?;
                if linked.is_some_and(|claim| claim.organization_id.is_some()) {
                    return install_link(state, principal.github_id(), true).await;
                }
            }
            attempt.direct
        }
        Some(InstallContext::AmbiguousAdd) => false,
        None => true,
    };
    install_link(state, principal.github_id(), direct).await
}

#[utoipa::path(operation_id="installation_links_start",post,path="/installation-links/start",responses((status=200,body=InstallLink)),security(("session"=[])))]
pub(crate) async fn start(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
) -> Result<Json<InstallLink>> {
    Ok(Json(continuation_link(&state, &principal).await?))
}

#[utoipa::path(operation_id="organizations_add",post,path="/workspaces/{workspace_id}/organizations/add",params(WorkspacePath),responses((status=200,body=InstallLink)),security(("session"=[])))]
pub(crate) async fn begin(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<WorkspacePath>,
) -> Result<Json<InstallLink>> {
    state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    Ok(Json(
        install_link(&state, principal.github_id(), false).await?,
    ))
}

#[utoipa::path(operation_id="installation_links_list",get,path="/installation-links",responses((status=200,body=[OrganizationLinkView])),security(("session"=[])))]
pub(crate) async fn list(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
) -> Result<Json<Vec<OrganizationLinkView>>> {
    let claims = state
        .access
        .refresh_installation_links(&principal, state.github.as_ref())
        .await?;
    let person = principal.github_id();
    let durable = !principal.browser_only();
    query(state.store, move |store| {
        claims
            .into_iter()
            .map(|claim| link_views::view(store, claim, person, durable))
            .collect()
    })
    .await
    .map(Json)
}

#[utoipa::path(operation_id="installation_links_detail",get,path="/installation-links/{claim_id}",params(ClaimPath),responses((status=200,body=OrganizationLinkView)),security(("session"=[])))]
pub(crate) async fn detail(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<ClaimPath>,
) -> Result<Json<OrganizationLinkView>> {
    let person = principal.github_id();
    let durable = !principal.browser_only();
    query(state.store, move |store| {
        let claim = store
            .installation_claim(path.claim_id)?
            .ok_or(ciproxy_store_interface::error::Error::NotFound)?;
        link_views::view(store, claim, person, durable)
    })
    .await
    .map(Json)
}

#[utoipa::path(operation_id="installation_links_choose",put,path="/installation-links/{claim_id}/workspace",params(ClaimPath),request_body=WorkspaceChoice,responses((status=200,body=OrganizationLinkView)),security(("session"=[])))]
pub(crate) async fn choose(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<ClaimPath>,
    Json(choice): Json<WorkspaceChoice>,
) -> Result<Json<OrganizationLinkView>> {
    state
        .access
        .choose_installation_link(
            &principal,
            path.claim_id,
            choice.workspace_id,
            state.github.as_ref(),
        )
        .await?;
    detail(State(state), Extension(principal), Path(path)).await
}

#[utoipa::path(operation_id="organizations_enable",post,path="/workspaces/{workspace_id}/organizations/{organization_id}/enable",params(OrganizationPath),responses((status=200,body=Success)),security(("session"=[])))]
pub(crate) async fn enable(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<OrganizationPath>,
) -> Result<Json<Success>> {
    let workspace = state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    query(state.store.clone(), move |store| {
        store.approve_organization_activation(workspace, path.organization_id)
    })
    .await?;
    state
        .github
        .reconcile_organization_activation(workspace)
        .await
        .api()?;
    Ok(Json(Success { ok: true }))
}
