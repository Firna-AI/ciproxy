//! Remote ownership markers and label restoration without reserved user names.

use ciproxy_store_interface::labels::Label;
use scaleset::{RunnerGroupId, RunnerScaleSet, RunnerScaleSetSpec, ScaleSetId};

use crate::error::{Result, ResultExt};

use super::failure::{Operation, missing};
use super::sdk::SdkScaleSetApi;

/// Request quarantine when ownership of one remote ID spans workspaces.
pub(crate) fn quarantine_name(github_id: i64) -> String {
    format!("ciproxy-quarantined-{github_id}")
}

fn quarantine_label(name: &str) -> String {
    format!("{name}-quarantine")
}

/// The paired label differs from the name, unlike an ordinary customer label.
pub(super) fn is_quarantined(set: &RunnerScaleSet) -> bool {
    set.labels
        .iter()
        .map(|label| label.name.as_str())
        .eq([quarantine_label(&set.name).as_str()])
}

/// Leave every requested customer label available during remote retirement.
pub(crate) fn retirement_name(base: &str, labels: &[Label]) -> String {
    available_name(base, &|name| {
        labels
            .iter()
            .any(|label| label.name == name || label.name == quarantine_label(name))
    })
}

fn available_name(base: &str, occupied: &dyn Fn(&str) -> bool) -> String {
    let mut name = base.to_owned();
    let mut suffix = 0;
    while occupied(&name) {
        suffix += 1;
        name = format!("{base}-{suffix}");
    }
    name
}

impl SdkScaleSetApi {
    /// Restore a safe saved resource after retirement in another workspace.
    pub(super) async fn restore_label(&self, set: &RunnerScaleSet, label: &str) -> Result<i64> {
        if set.name != label
            || !set
                .labels
                .iter()
                .map(|value| value.name.as_str())
                .eq([label])
        {
            self.client
                .update_runner_scale_set(set.id, &RunnerScaleSetSpec::new(label).labels([label]))
                .await
                .internal()?;
        }
        Ok(set.id.get())
    }

    /// Keep the quarantine pair when an internal resource name changes.
    pub(super) async fn retire_remote(&self, id: i64, name: &str) -> Result<()> {
        let existing = match self.client.get_runner_scale_set_by_id(ScaleSetId(id)).await {
            Ok(existing) => existing,
            Err(error) if missing(&error, Operation::ScaleSet(id)) => return Ok(()),
            Err(error) => return Err(error).internal(),
        };
        let quarantine = is_quarantined(&existing)
            || name == quarantine_name(id)
            || name.starts_with(&format!("{}-", quarantine_name(id)));
        let sets = self
            .client
            .list_runner_scale_sets(existing.runner_group_id)
            .await
            .internal()?;
        self.rename_retired(&existing, name, quarantine, &sets, None)
            .await
    }

    /// Move orphaned quarantine markers before a customer label can match them.
    pub(super) async fn release_quarantine_labels(
        &self,
        group: RunnerGroupId,
        label: &str,
    ) -> Result<()> {
        let sets = self.client.list_runner_scale_sets(group).await.internal()?;
        for set in sets.iter().filter(|set| {
            is_quarantined(set)
                && (set.name == label || set.labels.iter().any(|value| value.name == label))
        }) {
            self.rename_retired(
                set,
                &quarantine_name(set.id.get()),
                true,
                &sets,
                Some(label),
            )
            .await?;
        }
        Ok(())
    }

    async fn rename_retired(
        &self,
        set: &RunnerScaleSet,
        base: &str,
        quarantine: bool,
        sets: &[RunnerScaleSet],
        forbidden: Option<&str>,
    ) -> Result<()> {
        let name = available_name(base, &|name| {
            let marker = quarantine_label(name);
            forbidden.is_some_and(|label| label == name || label == marker)
                || sets.iter().filter(|other| other.id != set.id).any(|other| {
                    other.name == name
                        || other
                            .labels
                            .iter()
                            .any(|label| label.name == name || (quarantine && label.name == marker))
                })
        });
        let label = if quarantine {
            quarantine_label(&name)
        } else {
            name.clone()
        };
        match self
            .client
            .update_runner_scale_set(set.id, &RunnerScaleSetSpec::new(&name).labels([label]))
            .await
        {
            Ok(_) => Ok(()),
            Err(error) if missing(&error, Operation::ScaleSet(set.id.get())) => Ok(()),
            Err(error) => Err(error).internal(),
        }
    }
}
