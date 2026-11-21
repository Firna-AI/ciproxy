//! Shared runtime, job, and saved-total fixtures for metering integration.

use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use ciproxy_control_plane::accounting::AccountingWorker;
use ciproxy_control_plane::runtime::MachineRuntime;
use ciproxy_gateway::gateway::MachineGatewayMock;
use ciproxy_gateway::types::Delivery;
use ciproxy_github::assignments::AssignmentService;
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::runtime::{Runtime, RuntimeMock};
use ciproxy_github::timing::JobTimingMock;
use ciproxy_metering::service::StoredMetering;
use ciproxy_store_interface::github::{DemandPhase, GithubJob, RepositoryIdentity, RunFacts};
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use super::support::{Fixture, at};

pub(super) fn make_runtime(fixture: &Fixture) -> (MachineRuntime, AccountingWorker) {
    let clock: Arc<dyn Runtime> = Arc::new(Unimock::new((
        RuntimeMock::now
            .each_call(matching!())
            .returns(at() + Duration::minutes(5)),
        RuntimeMock::uuid
            .each_call(matching!())
            .returns(Uuid::from_u128(99)),
    )));
    let runtime = MachineRuntime::new(
        fixture.store.clone(),
        Arc::new(Unimock::new(
            MachineGatewayMock::send
                .each_call(matching!(_, _, _))
                .answers(&|_, _, _, _| Ok(Delivery::Connected)),
        )),
        Arc::new(Unimock::new(())),
        Arc::new(AssignmentService {
            store: Arc::new(StoreAdapter::new(fixture.store.clone(), clock.clone())),
        }),
        clock.clone(),
        None,
        Arc::new(Unimock::new(())),
    );
    let accounting = AccountingWorker::new(
        fixture.store.clone(),
        Arc::new(Unimock::new(
            JobTimingMock::repair
                .each_call(matching!(_))
                .answers(&|_, _| Ok(())),
        )),
        Arc::new(StoredMetering::new(fixture.store.clone())),
        clock,
    );
    (runtime, accounting)
}
pub(super) fn time(value: &str) -> DateTime<Utc> {
    value.parse().unwrap()
}
pub(super) fn job(fixture: &Fixture) -> GithubJob {
    GithubJob {
        id: Uuid::new_v4(),
        organization_id: fixture.organization,
        scale_set_id: fixture.intent.runner.scale_set_id,
        label_id: fixture.intent.runner.label_id,
        actions_job_id: None,
        request_id: 0,
        run_id: 7,
        repository: "org/repo".to_owned(),
        name: "build".to_owned(),
        workflow_ref: "ci.yml".to_owned(),
        phase: DemandPhase::Started,
        result: None,
        assigned_at: Some(time("2026-11-05T11:59:00.000000001Z")),
        started_at: Some(time("2026-11-05T12:00:00.000000001Z")),
        completed_at: None,
        runner_github_id: Some(47),
        facts: RunFacts {
            id: 7,
            repository: RepositoryIdentity {
                id: 8,
                full_name: "org/repo".to_owned(),
            },
            event: "push".to_owned(),
            head_branch: Some("main".to_owned()),
            head_sha: "fixture".to_owned(),
            head_repository: None,
        },
    }
}
pub(super) fn totals(fixture: &Fixture) -> (u64, String) {
    let day = at().date_naive();
    let snapshot = fixture
        .store
        .savings_snapshot(fixture.workspace, day, day.succ_opt().unwrap())
        .unwrap();
    (
        snapshot
            .days
            .iter()
            .flat_map(|day| &day.sizes)
            .map(|size| size.machines.starts)
            .sum(),
        snapshot
            .days
            .iter()
            .flat_map(|day| &day.sizes)
            .map(|size| size.machines.minutes)
            .sum::<rust_decimal::Decimal>()
            .normalize()
            .to_string(),
    )
}
