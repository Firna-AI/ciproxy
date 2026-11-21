//! Opaque HTTP workspace candidates opened only through current authority.

use ciproxy_store_interface::dashboard::records::Role;
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::state::query;

use super::{AccessService, Principal};

/// A valid wire UUID with no unguarded database-ID access for HTTP handlers.
#[derive(Clone, Copy, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct WorkspaceId(Uuid);

impl AccessService {
    pub(super) async fn open_workspace(
        &self,
        principal: &Principal,
        candidate: WorkspaceId,
        admin: bool,
    ) -> Result<Uuid> {
        if principal.pending {
            return Err(Error::Forbidden);
        }
        let id = candidate.0;
        let github = principal.github_id();
        let role = query(self.store.clone(), move |store| {
            store.dashboard_role(id, github)
        })
        .await?
        .ok_or(Error::Forbidden)?;
        if admin && role != Role::Admin {
            return Err(Error::Forbidden);
        }
        Ok(id)
    }

    pub(super) async fn open_owner_claim(
        &self,
        principal: &Principal,
        candidate: WorkspaceId,
    ) -> Result<Uuid> {
        if principal.pending {
            return Err(Error::Forbidden);
        }
        let id = candidate.0;
        let person = principal.id();
        let visible = query(self.store.clone(), move |store| {
            Ok(store
                .dashboard_workspaces(person)?
                .iter()
                .any(|workspace| workspace.id == id))
        })
        .await?;
        if !visible {
            return Err(Error::Forbidden);
        }
        Ok(id)
    }
}
