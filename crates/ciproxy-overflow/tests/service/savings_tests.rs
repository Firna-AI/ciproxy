//! Production Boat completion supplies the public savings report without a confirmed fee.

use chrono::Duration;
use ciproxy_metering::report::catalog::{Alternative, PriceList, SizePrice};
use ciproxy_metering::report::range::DateRange;
use ciproxy_metering::service::{Metering, StoredMetering};
use ciproxy_overflow::api::{Overflow, StartOutcome};
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::jobs::{Job, JobResult, JobState};
use ciproxy_store_interface::metering::history::PlanCost;
use ciproxy_store_interface::overflow::record::StartPhase;
use rust_decimal::Decimal;
use serde_json::json;

use crate::tls::facts::{identity, limits};
use crate::tls::server::{Server, step};

use super::authority_support::{DisableAt, Provider};
use super::fixtures::{at, id, request};
use super::harness::{Harness, Runtime};

#[tokio::test]
async fn measured_production_start_uses_saved_fee_in_the_public_savings_report() {
    let finished = at() + Duration::minutes(10);
    let usage = json!({"ok":true,"type":"sandbox.usage","sandboxId":"bx_23456789",
        "since":at(),"until":finished,"dollars":0.012,"running":false});
    let server = Server::new(vec![
        step("GET", "/api/v1/me", 200, identity()),
        step("GET", "/api/v1/limits", 200, limits(true)),
        step(
            "GET",
            "/api/v1/sandboxes/bx_23456789/usage?",
            200,
            usage.clone(),
        ),
        step("GET", "/api/v1/sandboxes/bx_23456789/usage?", 200, usage),
    ])
    .await;
    let provider = Provider::new(DisableAt::Never);
    let mut backend = provider.backend;
    backend.account = server.backend.account.clone();
    let harness = Harness::new(backend, (), Runtime::Start);
    let started = harness.service.start(id(1), request()).await.unwrap();
    assert!(matches!(started, StartOutcome::Started(_)));
    assert_eq!(harness.start(id(11)).start_usd, None);
    *harness.now.lock().unwrap() = finished;
    harness
        .service
        .report_end(Secret::new("callback-token-for-one-sandbox"), id(12))
        .await
        .unwrap();
    let record = harness.start(id(11));
    assert_eq!(record.phase, StartPhase::Ended);
    assert_eq!(record.start_usd, None);
    assert_eq!(record.start_fee_bound_usd, Some(Decimal::new(7, 2)));
    assert_eq!(record.runtime[0].usd, Decimal::new(12, 3));
    assert!(record.runtime_complete);
    harness.set_checked(finished);
    let view = harness.service.view(id(1)).await.unwrap();
    assert_eq!(view.spend.confirmed_usd, Decimal::new(12, 3));
    assert_eq!(view.spend.pending_usd, Decimal::new(7, 2));
    assert_eq!(view.scaler_account(finished).unwrap().spend.spent, 82_000);

    let store = &harness.database.store;
    store.record_workspace_start(id(1), at()).unwrap();
    store
        .save_plan_cost(
            id(1),
            &PlanCost {
                id: id(1),
                starts_at: at(),
                ends_at: None,
                monthly_usd: Decimal::ZERO,
            },
        )
        .unwrap();
    assert!(store.insert_job(id(1), &job()).unwrap());
    let metering: &dyn Metering = &StoredMetering::new(store.clone());
    let refreshed = metering.refresh(id(1), finished).unwrap();
    let report = metering.savings(
        id(1),
        DateRange {
            from: at().date_naive(),
            until: at().date_naive().succ_opt().unwrap(),
        },
        finished,
        &prices(),
    );
    assert!(
        report.is_ok(),
        "ordinary measured Boat use needs savings: {report:?}"
    );
    let report = report.unwrap();
    assert_eq!(report.costs.boat_usd, Decimal::new(82, 3));
    assert_eq!(report.costs.known_machines_usd, Decimal::ZERO);
    assert_eq!(report.costs.known_plan_usd, Decimal::ZERO);
    assert_eq!(report.costs.total_usd, Some(Decimal::new(82, 3)));
    assert_eq!(report.costs.missing, []);
    assert_eq!(refreshed.updated_jobs, 1);
    assert_eq!(refreshed.updated_sandboxes, 1);
    assert_eq!(report.boat.starts, 1);
    assert_eq!(report.boat.minutes, Decimal::from(10));
    assert_eq!(report.machines.starts, 0);
    assert_eq!(report.machines.minutes, Decimal::ZERO);
    assert_eq!(report.days.len(), 1);
    assert_eq!(report.days[0].sizes[0].boat.starts, 1);
    assert_eq!(report.days[0].sizes[0].boat.minutes, Decimal::from(10));
    assert_eq!(report.comparisons[0].usd, Some(Decimal::new(16, 2)));
    assert_eq!(report.comparisons[1].usd, Some(Decimal::new(32, 2)));
    assert_eq!(report.savings[0].usd, Some(Decimal::new(78, 3)));
    assert_eq!(report.savings[1].usd, Some(Decimal::new(238, 3)));
    assert_eq!(report.lead, Some(Alternative::Blacksmith));
    assert_eq!(harness.start(id(11)).start_usd, None);
    server.assert_consumed();
}

fn job() -> Job {
    Job {
        id: id(60),
        organization_id: id(3),
        github_id: 60,
        label_id: id(4),
        scale_set_id: id(5),
        runner_id: Some(id(12)),
        name: "Measured job".to_owned(),
        workflow: "CI".to_owned(),
        repository: "fixture/private".to_owned(),
        git_ref: None,
        state: JobState::Completed,
        result: Some(JobResult::Success),
        queued_at: at(),
        started_at: Some(at()),
        completed_at: Some(at() + Duration::minutes(10)),
        routing_reason: None,
        cache_source: None,
        machine_handled_cache_counts: None,
    }
}

fn prices() -> [PriceList; 2] {
    [
        (Alternative::Blacksmith, Decimal::new(16, 3)),
        (Alternative::Github, Decimal::new(32, 3)),
    ]
    .map(|(alternative, minute_usd)| PriceList {
        alternative,
        dated: at().date_naive(),
        sizes: vec![SizePrice {
            vcpus: 8,
            memory_gb: 16,
            minute_usd,
        }],
    })
}
