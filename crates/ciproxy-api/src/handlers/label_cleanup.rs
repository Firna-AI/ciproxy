//! Retired binding cleanup preserves completion sessions and replacement labels.

use ciproxy_store_interface::organizations::AppState;
use uuid::Uuid;

use crate::error::{ErrorResultExt, Result, ServiceResultExt};
use crate::state::{ApiState, query};

pub(crate) async fn cleanup(state: &ApiState, workspace: Uuid, label: Uuid) -> Result<()> {
    let organizations = query(state.store.clone(), move |store| {
        store.organizations(workspace)
    })
    .await?;
    for organization in organizations
        .into_iter()
        .filter(|organization| organization.app_state == AppState::Active)
    {
        let id = organization.github_id;
        let mut lease = query(state.store.clone(), move |store| {
            store.lock_github_organization(id)
        })
        .await?;
        let organization_id = organization.id;
        let sets = query(state.store.clone(), move |store| {
            store.dashboard_scale_sets(workspace, organization_id)
        })
        .await?;
        let api = state
            .scale_sets
            .for_organization(&organization)
            .await
            .api()?;
        for set in sets.into_iter().filter(|set| set.label_id == label) {
            let set_id = set.id;
            let binding = query(state.store.clone(), move |store| {
                store.scale_set_binding(workspace, organization_id, set_id)
            })
            .await?;
            let Some(binding) = binding else { continue };
            if binding.has_tracked_jobs {
                api.rename_scale_set(
                    set.github_id,
                    &format!("ciproxy-retired-{}", set.id.simple()),
                )
                .await
                .api()?;
                continue;
            }
            let lock = query(state.store.clone(), move |store| {
                store.try_lock_scale_set(workspace, organization_id, set_id)
            })
            .await?;
            let Some(mut lock) = lock else {
                api.rename_scale_set(
                    set.github_id,
                    &format!("ciproxy-retired-{}", set.id.simple()),
                )
                .await
                .api()?;
                continue;
            };
            let binding = query(state.store.clone(), move |store| {
                store.scale_set_binding(workspace, organization_id, set_id)
            })
            .await?;
            let Some(binding) = binding else { continue };
            if binding.has_tracked_jobs {
                api.rename_scale_set(
                    set.github_id,
                    &format!("ciproxy-retired-{}", set.id.simple()),
                )
                .await
                .api()?;
                continue;
            }
            lease.check_connection().internal()?;
            api.remove_scale_set(set.github_id).await.api()?;
            lock.check_connection().internal()?;
            lease.check_connection().internal()?;
            let at = state.runtime.now();
            query(state.store.clone(), move |store| {
                store.complete_scale_set_removal(workspace, &set, at)
            })
            .await?;
        }
    }
    Ok(())
}
