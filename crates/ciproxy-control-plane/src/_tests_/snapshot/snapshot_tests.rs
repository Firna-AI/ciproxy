//! Explicit host budgets and the documented history window change routing.

use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_protocol::identifiers::OrganizationId;
use ciproxy_scaler::actions::{Action, StartReason};

use super::fixtures::{at, input, job, organization};
use crate::snapshot::build;

#[test]
fn persisted_image_budget_still_limits_warm_starts_when_cache_reporting_fails() {
    let input = input();
    let runnable = input
        .organizations
        .iter()
        .map(|organization| organization.id)
        .collect::<Vec<_>>();
    let actions =
        ciproxy_scaler::decision::decide(&build(&input, &runnable, at(), RootSizing::default()));
    assert_eq!(actions.iter().filter(|action|matches!(action,Action::StartRunner(start) if start.reason==StartReason::WarmPool)).count(),1);
}

#[test]
fn warm_history_uses_only_default_label_jobs_in_the_last_day() {
    let mut input = input();
    input.organizations.push(organization(6));
    input.machines[0]
        .report
        .as_mut()
        .unwrap()
        .report
        .job_disk_bytes = 600_000_000_000;
    input.jobs = vec![
        job(10, 2, 25),
        job(11, 2, 25),
        job(12, 2, 25),
        job(13, 6, 1),
    ];
    let runnable = input
        .organizations
        .iter()
        .map(|organization| organization.id)
        .collect::<Vec<_>>();
    let actions =
        ciproxy_scaler::decision::decide(&build(&input, &runnable, at(), RootSizing::default()));
    let starts = actions
        .into_iter()
        .filter_map(|action| match action {
            Action::StartRunner(start) => Some(start.organization_id),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        starts,
        vec![
            OrganizationId(input.organizations[1].id.to_string()),
            OrganizationId(input.organizations[1].id.to_string())
        ]
    );
}

#[test]
fn older_selected_local_image_keeps_its_own_root_allocation_during_a_new_release_download() {
    let mut input = input();
    input.machines[0]
        .report
        .as_mut()
        .unwrap()
        .report
        .images
        .available[0]
        .root_disk_bytes = 1_000_000_000;
    let runnable = input
        .organizations
        .iter()
        .map(|organization| organization.id)
        .collect::<Vec<_>>();
    let snapshot = build(
        &input,
        &runnable,
        at(),
        RootSizing {
            image_root_bytes: 10_000_000_000,
        },
    );
    let starts: Vec<_> = ciproxy_scaler::decision::decide(&snapshot)
        .into_iter()
        .filter_map(|action| match action {
            Action::StartRunner(start) => Some(start),
            _ => None,
        })
        .collect();
    assert_eq!(starts.len(), 1);
    assert_eq!(starts[0].requested_size.disk_bytes, 80_000_000_000);
    assert_eq!(starts[0].size.disk_bytes, 83_491_524_608);
    let policy = crate::docker_budget::policy(&input, input.machines[0].machine.id).unwrap();
    assert_eq!(policy.image.root_disk_bytes, 1_000_000_000);
    assert_eq!(policy.spare_bytes, 0);
}
