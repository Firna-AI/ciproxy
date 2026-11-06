//! Credential-free mappings from store values to dashboard contract values.

use ciproxy_store_interface::dashboard::records::{Member, Role};
use ciproxy_store_interface::organizations::{AppState, Organization, RepositoryAccess};

use crate::error::{Error, Result};
use crate::wire::access::{MemberView, WorkspaceRole};
use crate::wire::settings::{InstallationState, OrganizationView, RepositorySelection};

pub(super) fn role(value: Role) -> WorkspaceRole {
    match value {
        Role::Admin => WorkspaceRole::Admin,
        Role::Billing => WorkspaceRole::Billing,
        Role::Viewer => WorkspaceRole::Viewer,
    }
}

pub(super) fn stored_role(value: WorkspaceRole) -> Role {
    match value {
        WorkspaceRole::Admin => Role::Admin,
        WorkspaceRole::Billing => Role::Billing,
        WorkspaceRole::Viewer => Role::Viewer,
    }
}

pub(super) fn member(value: Member) -> MemberView {
    MemberView {
        id: value.github_user_id.to_string(),
        login: value.login,
        role: value.role.map(role),
        requested_at: value.requested_at,
    }
}

pub(super) fn organization(value: Organization) -> OrganizationView {
    OrganizationView {
        id: value.id.to_string(),
        github_id: value.github_id.to_string(),
        login: value.login.clone(),
        installation_id: value.installation_id.to_string(),
        state: match value.app_state {
            AppState::Active => InstallationState::Active,
            AppState::Suspended => InstallationState::Suspended,
            AppState::Removed => InstallationState::Removed,
        },
        delete_after: value.delete_after,
        access: match value.repositories {
            RepositoryAccess::AllPrivate => RepositorySelection::AllPrivate,
            RepositoryAccess::Selected(ids) => RepositorySelection::Selected {
                ids: ids.iter().map(ToString::to_string).collect(),
            },
        },
        manage_url: format!(
            "https://github.com/organizations/{}/settings/installations/{}",
            value.login, value.installation_id
        ),
    }
}

pub(super) fn access(value: RepositorySelection) -> Result<RepositoryAccess> {
    match value {
        RepositorySelection::AllPrivate => Ok(RepositoryAccess::AllPrivate),
        RepositorySelection::Selected { ids } => {
            let mut parsed = Vec::new();
            for id in ids {
                match id.parse::<i64>() {
                    Ok(id) => parsed.push(id),
                    Err(_) => return Err(Error::InvalidInput),
                }
            }
            Ok(RepositoryAccess::Selected(parsed))
        }
    }
}
