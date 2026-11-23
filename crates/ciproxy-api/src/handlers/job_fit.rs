//! Waiting-label fit through the existing machine, image, and scaler owners.

use std::collections::BTreeMap;

use ciproxy_gateway::types::MachineView;
use ciproxy_store_interface::job_views::DashboardJob;
use ciproxy_store_interface::jobs::JobState;
use ciproxy_store_interface::labels::Label;
use uuid::Uuid;

use crate::capacity::MachineCapacity;
use crate::wire::job_fit::{ProviderSizeFit, WaitingLabelFit};
use crate::wire::settings::LabelPlace;

use super::{label_provider::Observation, label_views};

pub(super) struct FitInput<'a> {
    pub jobs: &'a [DashboardJob],
    pub labels: Vec<Label>,
    pub machines: &'a [MachineView],
    pub capacity: &'a dyn MachineCapacity,
    pub observation: &'a Observation<'a>,
    pub admin: bool,
}

pub(super) fn labels(input: FitInput<'_>) -> Vec<WaitingLabelFit> {
    let FitInput {
        jobs,
        labels,
        machines,
        capacity,
        observation,
        admin,
    } = input;
    let mut waiting: BTreeMap<Uuid, (String, u64)> = BTreeMap::new();
    for job in jobs.iter().filter(|job| job.state == JobState::Queued) {
        let entry = waiting
            .entry(job.label_id)
            .or_insert((job.label.clone(), 0));
        entry.1 += 1;
    }
    waiting
        .into_iter()
        .map(|(id, (name, count))| {
            let label = labels.iter().find(|label| label.id == id);
            let current = label
                .map(|label| label_views::label(label.clone(), machines, capacity, observation));
            let nothing_fits = current.as_ref().and_then(|current| {
                if current.fit.fitting_machines > 0
                    || current.fit.providers.iter().any(|provider| {
                        matches!(
                            provider.fit,
                            ProviderSizeFit::Fitting { .. }
                                | ProviderSizeFit::LargestFallback { .. }
                        )
                    })
                {
                    Some(false)
                } else if current.label.order.contains(&LabelPlace::Machine)
                    && current.fit.unavailable_machines > 0
                    || current
                        .fit
                        .providers
                        .iter()
                        .any(|provider| matches!(provider.fit, ProviderSizeFit::Unavailable))
                {
                    None
                } else {
                    Some(true)
                }
            });
            let providers = current
                .as_ref()
                .map(|current| current.fit.providers.clone())
                .unwrap_or_default();
            WaitingLabelFit {
                label_id: id.to_string(),
                label_name: name,
                waiting_jobs: count.to_string(),
                can_edit: admin && current.is_some(),
                ask_admin: !admin && nothing_fits == Some(true),
                current,
                providers,
                nothing_fits,
            }
        })
        .collect()
}
