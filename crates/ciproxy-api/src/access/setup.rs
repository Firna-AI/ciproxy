//! Setup proof joins configured-App identity with fresh user authority.

use ciproxy_github::app::webhook::AccountKind;
use ciproxy_github::installation_state::InstalledApp;
use ciproxy_store_interface::dashboard::records::Membership;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::identity::{Grant, Profile};
use crate::wire::access::SetupOutcome;

use super::AccessService;

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum AccountType {
    Organization,
    User,
}

#[derive(Deserialize, Serialize)]
pub(super) struct SetupTarget {
    installation_id: i64,
    account_id: i64,
    kind: AccountType,
}

impl SetupTarget {
    pub(super) fn configured(installation: InstalledApp) -> Result<Self> {
        let kind = match installation.account.kind {
            AccountKind::Organization => AccountType::Organization,
            AccountKind::User => AccountType::User,
            AccountKind::Other => return Err(Error::InvalidInput),
        };
        Ok(Self {
            installation_id: installation.id,
            account_id: installation.account.id,
            kind,
        })
    }
}

impl AccessService {
    pub(super) async fn prove_setup(
        &self,
        target: &SetupTarget,
        grant: &Grant,
        profile: &Profile,
        memberships: &[Membership],
    ) -> Result<SetupOutcome> {
        let actual = self
            .identity
            .installation(grant, target.installation_id)
            .await?
            .ok_or(Error::Forbidden)?;
        if actual.id != target.installation_id || actual.account.id != target.account_id {
            return Err(Error::Forbidden);
        }
        match (target.kind, actual.account.kind) {
            (AccountType::Organization, AccountKind::Organization) => {
                if !memberships
                    .iter()
                    .any(|membership| membership.github_id == actual.account.id)
                {
                    return Err(Error::Forbidden);
                }
                Ok(SetupOutcome::Organization {
                    installation_id: actual.id.to_string(),
                    github_id: actual.account.id.to_string(),
                    login: actual.account.login,
                    suspended: actual.suspended,
                })
            }
            (AccountType::User, AccountKind::User) if actual.account.id == profile.id => {
                Ok(SetupOutcome::OrganizationsOnly {
                    installation_id: actual.id.to_string(),
                    remove_url: format!("https://github.com/settings/installations/{}", actual.id),
                })
            }
            _ => Err(Error::Forbidden),
        }
    }
}
