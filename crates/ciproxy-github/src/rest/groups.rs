//! Dedicated runner-group reconciliation without editing existing groups.

use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize};

use crate::error::{Result, ResultExt};

use super::http::RestClient;

#[derive(Deserialize)]
struct Groups {
    runner_groups: Vec<Group>,
}

#[derive(Deserialize)]
struct Group {
    id: i64,
    name: String,
    inherited: bool,
}

#[derive(Serialize)]
struct GroupPolicy<'a> {
    name: &'a str,
    visibility: &'static str,
    allows_public_repositories: bool,
    selected_repository_ids: &'a [i64],
}

#[derive(Serialize)]
struct Selected<'a> {
    selected_repository_ids: &'a [i64],
}

impl RestClient {
    pub(super) async fn sync_group(
        &self,
        installation_id: i64,
        organization: &str,
        name: &str,
        repository_ids: &[i64],
        saved_id: Option<i64>,
    ) -> Result<i64> {
        let path = format!("orgs/{organization}/actions/runner-groups");
        let mut page = 1;
        let mut existing = None;
        if let Some(id) = saved_id {
            let response = self
                .request(installation_id, Method::GET, &format!("{path}/{id}"), &())
                .await?;
            if response.status() != StatusCode::NOT_FOUND {
                let group: Group = Self::success(response)?.json().await.internal()?;
                if !group.inherited {
                    existing = Some(group.id);
                }
            }
        }
        while existing.is_none() {
            let response = Self::success(
                self.request(
                    installation_id,
                    Method::GET,
                    &format!("{path}?per_page=100&page={page}"),
                    &(),
                )
                .await?,
            )?;
            let groups: Groups = response.json().await.internal()?;
            let done = groups.runner_groups.len() < 100;
            for group in groups.runner_groups {
                if group.name == name && !group.inherited {
                    existing = Some(group.id);
                }
            }
            if existing.is_some() || done {
                break;
            }
            page += 1;
        }
        let policy = GroupPolicy {
            name,
            visibility: "selected",
            allows_public_repositories: false,
            selected_repository_ids: repository_ids,
        };
        let id = if let Some(id) = existing {
            Self::success(
                self.request(
                    installation_id,
                    Method::PATCH,
                    &format!("{path}/{id}"),
                    &policy,
                )
                .await?,
            )?;
            id
        } else {
            let response = Self::success(
                self.request(installation_id, Method::POST, &path, &policy)
                    .await?,
            )?;
            let group: Group = response.json().await.internal()?;
            group.id
        };
        Self::success(
            self.request(
                installation_id,
                Method::PUT,
                &format!("{path}/{id}/repositories"),
                &Selected {
                    selected_repository_ids: repository_ids,
                },
            )
            .await?,
        )?;
        Ok(id)
    }
}
