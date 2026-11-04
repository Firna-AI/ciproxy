//! Private organization rows and typed repository selection mappings.

use chrono::{DateTime, Utc};
use ciproxy_schema::organizations::organizations;
use ciproxy_store_interface::error::{Error, Result, ResultExt};
use ciproxy_store_interface::organizations::{Organization, RepositoryAccess};
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::InternalCause;

#[derive(Queryable, Selectable)]
#[diesel(table_name = organizations)]
pub(crate) struct OrganizationRow {
    id: Uuid,
    workspace_id: Uuid,
    github_id: i64,
    login: String,
    installation_id: i64,
    app_state: String,
    repository_mode: String,
    allowed_repository_ids: Vec<i64>,
    removed_at: Option<DateTime<Utc>>,
    delete_after: Option<DateTime<Utc>>,
}

impl OrganizationRow {
    pub(crate) fn into_record(self) -> Result<Organization> {
        let repositories = match self.repository_mode.as_str() {
            "all_private" => RepositoryAccess::AllPrivate,
            "selected" => RepositoryAccess::Selected(self.allowed_repository_ids),
            _ => return Err(Error::internal(InternalCause::InvalidRow)),
        };
        Ok(Organization {
            id: self.id,
            workspace_id: self.workspace_id,
            github_id: self.github_id,
            login: self.login,
            installation_id: self.installation_id,
            app_state: self.app_state.parse().internal()?,
            repositories,
            removed_at: self.removed_at,
            delete_after: self.delete_after,
        })
    }
}

pub(crate) fn repository_columns(repositories: &RepositoryAccess) -> (&'static str, Vec<i64>) {
    match repositories {
        RepositoryAccess::AllPrivate => ("all_private", Vec::new()),
        RepositoryAccess::Selected(ids) => ("selected", ids.clone()),
    }
}
