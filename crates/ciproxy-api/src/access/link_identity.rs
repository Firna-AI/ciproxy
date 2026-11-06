//! Fresh link authority while retaining rotated grants before any membership read.

use chrono::Duration;
use ciproxy_store_interface::dashboard::records::{Membership, User};

use crate::error::{Error, ErrorResultExt, Result};
use crate::identity::Grant;
use crate::state::query;

use super::AccessService;

impl AccessService {
    /// Release the identity lease before the caller enters App organization work.
    pub(super) async fn refresh_link_identity(
        &self,
        user: User,
    ) -> Result<(Grant, Vec<Membership>)> {
        let person = user.github_id;
        let mut lease = query(self.store.clone(), move |store| {
            store.lock_github_organization(person)
        })
        .await?;
        let mut user = query(self.store.clone(), move |store| {
            store.dashboard_user_by_github(person)
        })
        .await?
        .ok_or(Error::Unauthorized)?;
        let now = self.runtime.now();
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
        let observed = memberships.clone();
        query(self.store.clone(), move |store| {
            store.save_dashboard_user(&user, &observed)
        })
        .await?;
        Ok((grant, memberships))
    }
}
