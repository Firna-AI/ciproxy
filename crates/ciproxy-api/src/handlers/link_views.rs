//! Typed requester recovery and local activation views.

use ciproxy_store_interface::billing::state::Tier;
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::error::{Error, Result};
use ciproxy_store_interface::installation_links::{
    InstallationClaim, InstallationRecovery, OrganizationActivation,
};
use ciproxy_store_interface::organizations::AppState;
use ciproxy_store_interface::store::Store;

use crate::wire::links::{
    ActivationReason, ActivationView, EligibleWorkspace, LinkState, OrganizationLinkView,
    OrganizationTier, OrganizationUpgrade,
};
use uuid::Uuid;

pub(super) fn activation(
    local: OrganizationActivation,
    workspace: Uuid,
    role: Option<Role>,
) -> ActivationView {
    let admin = role == Some(Role::Admin);
    let payer = matches!(role, Some(Role::Admin | Role::Billing));
    let mut reasons = Vec::new();
    if local.billing_hold {
        reasons.push(ActivationReason::BillingUpgrade);
    }
    if local.admin_approval_hold {
        reasons.push(ActivationReason::AdminApproval);
    }
    ActivationView {
        admitted: local.admitted,
        preparing: !local.admitted && reasons.is_empty(),
        reasons,
        added_by: local.added_by.map(|id| id.to_string()),
        can_enable: admin && local.admin_approval_hold && !local.billing_hold,
        upgrade: (payer && local.billing_hold).then_some(OrganizationUpgrade {
            workspace_id: workspace,
            tier: OrganizationTier::Team,
        }),
        ask_admin: (local.billing_hold && !payer) || (local.admin_approval_hold && !admin),
    }
}

pub(super) fn view(
    store: &dyn Store,
    claim: InstallationClaim,
    person: i64,
    durable: bool,
) -> Result<OrganizationLinkView> {
    if claim.requester_id != person {
        return Err(Error::NotFound);
    }
    if !durable {
        let waiting = claim.event_seen
            && claim.user_verified
            && claim.workspace_id.is_none()
            && store
                .installation_workspace_offers(claim.id, person)?
                .is_empty();
        return Ok(OrganizationLinkView {
            reauthorization_required: true,
            id: claim.id,
            installation_id: claim.installation_id.to_string(),
            organization: claim.login,
            state: if waiting {
                LinkState::WaitingForAdmin
            } else {
                LinkState::ReauthorizationRequired
            },
            workspaces: Vec::new(),
            workspace_id: None,
            organization_id: None,
            activation: None,
            reauthorization_url: "/api/auth/github".to_owned(),
        });
    }
    let mut workspaces = Vec::new();
    for workspace in store.installation_workspace_offers(claim.id, person)? {
        if store.dashboard_role(workspace, person)? != Some(Role::Admin) {
            continue;
        }
        if let Some(record) = store.workspace(workspace)? {
            let count = store
                .organizations(workspace)?
                .iter()
                .filter(|organization| organization.app_state != AppState::Removed)
                .count();
            workspaces.push(EligibleWorkspace {
                id: workspace,
                name: record.name,
                requires_upgrade: store.billing(workspace)?.state.tier == Tier::Free && count >= 1,
            });
        }
    }
    let local = match (claim.workspace_id, claim.organization_id) {
        (Some(workspace), Some(organization)) => Some(activation(
            store.organization_activation(workspace, organization)?,
            workspace,
            store.dashboard_role(workspace, person)?,
        )),
        _ => None,
    };
    let recovery = claim.recovery();
    let state = if let Some(local) = &local {
        if local.admitted {
            LinkState::Linked
        } else {
            LinkState::LinkedInactive
        }
    } else {
        match recovery {
            InstallationRecovery::DirectAwaitingEvent | InstallationRecovery::AddAwaitingEvent => {
                LinkState::ConfirmingInstallation
            }
            InstallationRecovery::AddAwaitingProof => LinkState::ReauthorizationRequired,
            InstallationRecovery::Direct | InstallationRecovery::Add { .. } => LinkState::Linking,
            InstallationRecovery::AddAwaitingWorkspace if workspaces.is_empty() => {
                LinkState::WaitingForAdmin
            }
            InstallationRecovery::AddAwaitingWorkspace => LinkState::ChooseWorkspace,
        }
    };
    Ok(OrganizationLinkView {
        reauthorization_required: !claim.user_verified
            && !matches!(
                recovery,
                InstallationRecovery::DirectAwaitingEvent | InstallationRecovery::Direct
            ),
        id: claim.id,
        installation_id: claim.installation_id.to_string(),
        organization: claim.login,
        state,
        workspaces,
        workspace_id: claim.workspace_id,
        organization_id: claim.organization_id,
        activation: local,
        reauthorization_url: "/api/auth/github".to_owned(),
    })
}
