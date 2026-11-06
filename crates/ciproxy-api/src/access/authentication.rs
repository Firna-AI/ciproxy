//! Stored session authentication and complete hourly membership refresh.

use async_trait::async_trait;
use chrono::Duration;
use ciproxy_github::app::GitHubApp;
use ciproxy_github::installation_state::InstalledApp;
use ciproxy_store_interface::dashboard::records::{Role, User};
use ciproxy_store_interface::installation_links::InstallationClaim;
use secrecy::SecretString;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::{Error, ErrorResultExt, Result};
use crate::identity::Grant;
use crate::state::query;

use super::flow::Purpose;
use super::setup::SetupTarget;
use super::workspace::WorkspaceId;
use super::{AccessService, DashboardAccess, LoginAttempt, LoginSuccess, Principal};

pub(crate) fn digest(value: &str) -> Vec<u8> {
    Sha256::digest(value.as_bytes()).to_vec()
}

impl AccessService {
    pub(super) async fn refresh_user(&self, mut user: User) -> Result<User> {
        let now = self.runtime.now();
        if now < user.checked_at + Duration::hours(1) {
            return Ok(user);
        }
        let github_id = user.github_id;
        let mut lease = query(self.store.clone(), move |store| {
            store.lock_github_organization(github_id)
        })
        .await?;
        user = query(self.store.clone(), move |store| {
            store.dashboard_user_by_github(github_id)
        })
        .await?
        .ok_or(Error::Unauthorized)?;
        if now < user.checked_at + Duration::hours(1) {
            return Ok(user);
        }
        let mut grant = Grant::decode(&self.cipher.open(user.id, user.credentials.clone()).await?)?;
        if grant
            .expires_at
            .is_some_and(|at| now + Duration::minutes(1) >= at)
        {
            grant = self.identity.refresh(grant).await?;
            user.credentials = self.cipher.seal(user.id, grant.encode()?).await?;
            lease.check_connection().internal()?;
            let id = user.id;
            let credentials = user.credentials.clone();
            query(self.store.clone(), move |store| {
                store.save_dashboard_credentials(id, &credentials)
            })
            .await?;
        }
        let memberships = self.identity.memberships(&grant).await?;
        user.credentials = self.cipher.seal(user.id, grant.encode()?).await?;
        user.checked_at = now;
        lease.check_connection().internal()?;
        let saved = user.clone();
        query(self.store.clone(), move |store| {
            store.save_dashboard_user(&saved, &memberships)
        })
        .await?;
        let id = user.id;
        query(self.store.clone(), move |store| store.dashboard_user(id))
            .await?
            .ok_or(Error::Unauthorized)
    }
}

#[async_trait]
impl DashboardAccess for AccessService {
    async fn begin_login(&self, previous: Option<&str>) -> Result<LoginAttempt> {
        self.start(Purpose::SignIn, previous).await
    }
    async fn begin_setup(
        &self,
        installation: Option<InstalledApp>,
        install_state: Option<String>,
        request_only: bool,
        previous: Option<&str>,
    ) -> Result<LoginAttempt> {
        self.start(
            Purpose::Setup {
                installation: installation.map(SetupTarget::configured).transpose()?,
                install_state,
                request_only,
            },
            previous,
        )
        .await
    }
    async fn finish_login(
        &self,
        code: SecretString,
        state: &str,
        cookie: &str,
        github: &dyn GitHubApp,
    ) -> Result<LoginSuccess> {
        self.finish(code, state, cookie, github).await
    }
    async fn refresh_installation_links(
        &self,
        principal: &Principal,
        github: &dyn GitHubApp,
    ) -> Result<Vec<InstallationClaim>> {
        self.refresh_links(principal, github).await
    }
    async fn choose_installation_link(
        &self,
        principal: &Principal,
        claim: Uuid,
        workspace: Uuid,
        github: &dyn GitHubApp,
    ) -> Result<()> {
        self.choose_link(principal, claim, workspace, github).await
    }
    async fn authenticate(&self, cookie: &str) -> Result<Principal> {
        let browser = self.open_browser(cookie).await?;
        let hash = digest(&browser.secret);
        let now = self.runtime.now();
        let session = query(self.store.clone(), move |store| {
            store.dashboard_session(&hash, now)
        })
        .await?;
        let Some(session) = session.filter(|session| {
            session.id == browser.session_id && session.user_id == browser.user_id
        }) else {
            return Ok(browser.principal(None));
        };
        let id = session.user_id;
        let user = query(self.store.clone(), move |store| store.dashboard_user(id)).await?;
        let Some(user) = user.filter(|user| user.github_id == browser.github_id) else {
            return Ok(browser.principal(None));
        };
        match self.refresh_user(user).await {
            Ok(user) => Ok(browser.principal(Some(user))),
            Err(Error::Unauthorized) => Ok(browser.principal(None)),
            Err(error) => Err(error),
        }
    }
    async fn sign_out(&self, principal: &Principal) -> Result<()> {
        if principal.pending {
            return Ok(());
        }
        let id = principal.session.id;
        query(self.store.clone(), move |store| {
            store.delete_dashboard_session(id)
        })
        .await
    }
    async fn reconcile(&self) -> Result<()> {
        let before = self.runtime.now() - Duration::hours(1);
        let users = query(self.store.clone(), move |store| {
            store.dashboard_users_due(before)
        })
        .await?;
        let mut failure = None;
        for user in users {
            if let Err(error) = self.refresh_user(user).await
                && !matches!(error, Error::Unauthorized)
            {
                failure = Some(error);
            }
        }
        let now = self.runtime.now();
        query(self.store.clone(), move |store| {
            store.expire_dashboard_auth(now)
        })
        .await?;
        if let Some(error) = failure {
            return Err(error);
        }
        Ok(())
    }
    async fn refresh_workspace_admins(&self, workspace: Uuid) -> Result<()> {
        let users = query(self.store.clone(), move |store| {
            let mut users = Vec::new();
            for member in store.dashboard_members(workspace)? {
                if member.role == Some(Role::Admin)
                    && let Some(user) = store.dashboard_user_by_github(member.github_user_id)?
                {
                    users.push(user);
                }
            }
            Ok(users)
        })
        .await?;
        for user in users {
            if let Err(error) = self.refresh_user(user).await
                && !matches!(error, Error::Unauthorized)
            {
                return Err(error);
            }
        }
        Ok(())
    }
    async fn require_role(
        &self,
        principal: &Principal,
        workspace_id: WorkspaceId,
        admin: bool,
    ) -> Result<Uuid> {
        self.open_workspace(principal, workspace_id, admin).await
    }
    async fn open_claim_workspace(
        &self,
        principal: &Principal,
        workspace_id: WorkspaceId,
    ) -> Result<Uuid> {
        self.open_owner_claim(principal, workspace_id).await
    }
}
