//! Waiting-label fit through the existing machine, image, and scaler owners.

use std::collections::BTreeMap;

use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_gateway::types::MachineView;
use ciproxy_overflow::view::ProviderView;
use ciproxy_store_interface::job_views::DashboardJob;
use ciproxy_store_interface::jobs::JobState;
use ciproxy_store_interface::labels::Label;
use uuid::Uuid;

use crate::capacity::MachineCapacity;
use crate::wire::job_fit::{BoatLabelFit, WaitingLabelFit};

use super::{label_boat, label_views};

pub(super) struct FitInput<'a> {
    pub jobs: &'a [DashboardJob],
    pub labels: Vec<Label>,
    pub machines: &'a [MachineView],
    pub capacity: &'a dyn MachineCapacity,
    pub overflow: Option<&'a ProviderView>,
    pub sizing: Option<RootSizing>,
    pub admin: bool,
}

pub(super) fn labels(input: FitInput<'_>) -> Vec<WaitingLabelFit> {
    let FitInput {
        jobs,
        labels,
        machines,
        capacity,
        overflow,
        sizing,
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
            let current = label.map(|label| {
                let mut current = label_views::label(label.clone(), machines, capacity);
                if let Some(sizing) = sizing {
                    label_boat::enrich(&mut current.fit, &current.label, overflow, sizing);
                }
                current
            });
            let own_fits = current
                .as_ref()
                .is_some_and(|current| current.fit.fitting_machines > 0);
            let own_unknown = current.as_ref().is_none_or(|current| {
                current.label.allow_machines && current.fit.unavailable_machines > 0
            });
            let boat = current
                .as_ref()
                .map(|current| current.fit.boat.clone())
                .unwrap_or(BoatLabelFit::Unavailable);
            let nothing_fits = if own_fits
                || matches!(
                    boat,
                    BoatLabelFit::Fitting { .. } | BoatLabelFit::LargestFallback { .. }
                ) {
                Some(false)
            } else if !own_unknown
                && matches!(boat, BoatLabelFit::NoSizeFits | BoatLabelFit::Disabled)
            {
                Some(true)
            } else {
                None
            };
            WaitingLabelFit {
                label_id: id.to_string(),
                label_name: name,
                waiting_jobs: count.to_string(),
                can_edit: admin && current.is_some(),
                ask_admin: !admin && nothing_fits == Some(true),
                current,
                boat,
                nothing_fits,
            }
        })
        .collect()
}
