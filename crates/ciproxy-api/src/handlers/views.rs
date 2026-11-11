//! Credential-free mappings from store values to dashboard contract values.

use ciproxy_store_interface::dashboard::records::{Member, Role};
use ciproxy_store_interface::installation_links::OrganizationActivation;
use ciproxy_store_interface::organizations::{AppState, Organization, RepositoryAccess};
use ciproxy_store_interface::store::Store;
use uuid::Uuid;

use super::link_views;
use crate::error::{Error, Result};
use crate::wire::access::{MemberView, WorkspaceRole};
use crate::wire::facts::{MemberOrganization, MemberProfile, OrganizationRepositories};
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
        name: value.name,
        avatar_url: value.avatar_url,
        granted_at: value.granted_at,
        request_organizations: value.request_organizations.map(|organizations| {
            organizations
                .into_iter()
                .map(|organization| MemberOrganization {
                    id: organization.id.to_string(),
                    login: organization.login,
                })
                .collect()
        }),
        role: value.role.map(role),
        requested_at: value.requested_at,
    }
}

pub(super) fn organization(value: Organization, local: OrganizationActivation) -> OrganizationView {
    OrganizationView {
        joined_at: None,
        removed_at: value.removed_at,
        repositories: None,
        added_by: local.added_by.map(|id| MemberProfile {
            id: id.to_string(),
            login: None,
            name: None,
            avatar_url: None,
        }),
        activation: link_views::activation(local, value.workspace_id, Some(Role::Admin)),
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

pub(super) fn organizations(
    store: &dyn Store,
    workspace: Uuid,
) -> ciproxy_store_interface::error::Result<Vec<OrganizationView>> {
    organization_records(store, workspace, store.organizations(workspace)?)
}

/// Map the already-read registration snapshot without another organization read.
pub(super) fn organization_records(
    store: &dyn Store,
    workspace: Uuid,
    organizations: Vec<Organization>,
) -> ciproxy_store_interface::error::Result<Vec<OrganizationView>> {
    organizations
        .into_iter()
        .map(|value| {
            let local = store.organization_activation(workspace, value.id)?;
            let facts = store.dashboard_organization_facts(workspace, value.id)?;
            let repositories = match facts.repositories_observed_at {
                Some(observed_at) => {
                    let repositories = store.repositories(workspace, value.id)?;
                    let private = repositories
                        .iter()
                        .filter(|repository| repository.private)
                        .count();
                    let allowed = repositories
                        .iter()
                        .filter(|repository| {
                            repository.private
                                && match &value.repositories {
                                    RepositoryAccess::AllPrivate => true,
                                    RepositoryAccess::Selected(ids) => ids.contains(&repository.id),
                                }
                        })
                        .count();
                    Some(OrganizationRepositories {
                        visible: repositories.len().to_string(),
                        private: private.to_string(),
                        allowed_private: allowed.to_string(),
                        observed_at,
                    })
                }
                None => None,
            };
            let adder = match local.added_by {
                Some(id) => store.dashboard_member_profile(id)?,
                None => None,
            };
            let mut view = organization(value, local);
            view.joined_at = facts.joined_at;
            view.repositories = repositories;
            if let (Some(profile), Some(adder)) = (view.added_by.as_mut(), adder) {
                profile.login = Some(adder.login);
                profile.avatar_url = Some(adder.avatar_url);
                profile.name = adder.name;
            }
            Ok(view)
        })
        .collect()
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
