//! Real stored histories and a trusted adjustable accounting clock.

use std::sync::{Arc, Mutex};

use chrono::{DateTime, Duration, Utc};
use ciproxy_control_plane::accounting::AccountingWorker;
use ciproxy_github::runtime::RuntimeMock;
use ciproxy_github::timing::JobTimingMock;
use ciproxy_metering::service::Metering;
use ciproxy_store_interface::github::{DemandPhase, GithubJob};
use ciproxy_store_interface::jobs::{Job, JobResult, JobState};
use ciproxy_store_interface::labels::Label;
use ciproxy_store_interface::metering::history::BoatCost;
use ciproxy_store_interface::metering::totals::{
    DailyTotal, LocationTotals, SizeTotal, StartTotal,
};
use ciproxy_store_interface::organizations::Organization;
use ciproxy_store_interface::runners::{Runner, RunnerPlace, RunnerState};
use ciproxy_store_interface::scale_sets::ScaleSet;
use ciproxy_store_interface::workspaces::Workspace;
use rust_decimal::Decimal;
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use super::recovery_support::job;
use super::support::{Fixture, at};

pub(super) struct History {
    pub(super) organization: Organization,
    pub(super) label: Label,
    pub(super) scale_set: ScaleSet,
    pub(super) runner: Runner,
    pub(super) job: Job,
}

pub(super) fn organization(fixture: &Fixture, workspace: Uuid, github_id: i64) -> Organization {
    if workspace != fixture.workspace {
        fixture
            .store
            .create_workspace(&Workspace {
                id: workspace,
                name: format!("Retention {github_id}"),
                delete_after: None,
            })
            .unwrap();
    }
    let mut record = fixture
        .store
        .organization(fixture.workspace, fixture.organization)
        .unwrap()
        .unwrap();
    record.id = Uuid::new_v4();
    record.workspace_id = workspace;
    record.github_id = github_id;
    record.installation_id = github_id + 1;
    record.login = format!("retention-{github_id}");
    fixture.store.install_organization(&record).unwrap();
    record
}

pub(super) fn history(fixture: &Fixture, organization: Organization, boat: bool) -> History {
    let workspace = organization.workspace_id;
    let store = &fixture.store;
    let mut label = store.labels(fixture.workspace).unwrap().remove(0);
    if workspace != fixture.workspace {
        label.id = Uuid::new_v4();
        store.create_label(workspace, &label).unwrap();
    }
    let scale_set = store
        .scale_sets(workspace, organization.id)
        .unwrap()
        .into_iter()
        .find(|scale_set| scale_set.label_id == label.id)
        .unwrap_or_else(|| {
            let scale_set = ScaleSet {
                id: Uuid::new_v4(),
                organization_id: organization.id,
                label_id: label.id,
                github_id: organization.github_id + 1_000,
                runner_group_id: organization.github_id + 2_000,
            };
            store.create_scale_set(workspace, &scale_set).unwrap();
            scale_set
        });
    let runner = Runner {
        id: Uuid::new_v4(),
        organization_id: organization.id,
        label_id: label.id,
        scale_set_id: scale_set.id,
        github_id: Some(organization.github_id + 3_000),
        state: RunnerState::Stopped,
        size: fixture.intent.runner.size,
        place: if boat {
            RunnerPlace::Boat {
                key_id: None,
                sandbox_id: format!("retention-{}", organization.github_id),
                size_name: "2cpu".to_owned(),
            }
        } else {
            fixture.intent.runner.place.clone()
        },
    };
    store.create_runner(workspace, &runner).unwrap();
    let started_at = at() + Duration::hours(i64::from(!boat));
    let job = Job {
        id: Uuid::new_v4(),
        organization_id: organization.id,
        github_id: organization.github_id + 4_000,
        label_id: label.id,
        scale_set_id: scale_set.id,
        runner_id: Some(runner.id),
        name: "retained job".to_owned(),
        workflow: "CI".to_owned(),
        repository: format!("{}/private", organization.login),
        git_ref: Some("refs/heads/main".to_owned()),
        state: JobState::Completed,
        result: Some(JobResult::Success),
        queued_at: started_at - Duration::minutes(1),
        started_at: Some(started_at),
        completed_at: Some(started_at + Duration::minutes(if boat { 5 } else { 7 })),
        routing_reason: None,
        cache_source: None,
        machine_handled_cache_counts: None,
    };
    store.insert_job(workspace, &job).unwrap();
    if boat {
        store
            .save_boat_cost(workspace, &cost(runner.id, 10))
            .unwrap();
    }
    History {
        organization,
        label,
        scale_set,
        runner,
        job,
    }
}

pub(super) fn cost(runner_id: Uuid, minutes: i64) -> BoatCost {
    BoatCost {
        runner_id,
        starts_at: at(),
        ends_at: Some(at() + Duration::minutes(minutes)),
        start_usd: Decimal::new(1, 1),
        minute_usd: Decimal::new(2, 2),
    }
}

pub(super) fn expired_demand(fixture: &Fixture) -> GithubJob {
    let mut old = job(fixture);
    old.phase = DemandPhase::Completed;
    old.runner_github_id = None;
    old.assigned_at = Some(at() - Duration::days(100));
    old.started_at = old.assigned_at;
    old.completed_at = Some(at() - Duration::days(100) + Duration::minutes(5));
    fixture
        .store
        .save_github_job(fixture.workspace, &old, at())
        .unwrap();
    old
}

pub(super) fn accounting(
    fixture: &Fixture,
    metering: Arc<dyn Metering>,
) -> (AccountingWorker, Arc<Mutex<DateTime<Utc>>>) {
    let now = Arc::new(Mutex::new(at()));
    let observed = now.clone();
    let worker = AccountingWorker::new(
        fixture.store.clone(),
        Arc::new(Unimock::new(
            JobTimingMock::repair
                .each_call(matching!(_))
                .answers(&|_, _| Ok(())),
        )),
        metering,
        Arc::new(Unimock::new(
            RuntimeMock::now
                .each_call(matching!())
                .answers_arc(Arc::new(move |_| *observed.lock().unwrap())),
        )),
    );
    (worker, now)
}

pub(super) fn expected_totals(fixture: &Fixture, machines: bool) -> Vec<DailyTotal> {
    vec![DailyTotal {
        day: at().date_naive(),
        sizes: vec![SizeTotal {
            size: fixture.intent.runner.size,
            machines: if machines {
                LocationTotals {
                    starts: 1,
                    minutes: Decimal::from(7),
                }
            } else {
                LocationTotals::default()
            },
            boat: LocationTotals {
                starts: 1,
                minutes: Decimal::from(5),
            },
        }],
        boat_usd: Decimal::new(3, 1),
    }]
}

pub(super) fn expected_starts(machines: bool) -> Vec<StartTotal> {
    let mut starts = vec![StartTotal {
        taken_at: at(),
        starts: 1,
    }];
    if machines {
        starts.push(StartTotal {
            taken_at: at() + Duration::hours(1),
            starts: 1,
        });
    }
    starts
}
