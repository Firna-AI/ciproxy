//! Savings joins provider usage and idle sandbox costs across retained days.

use chrono::Duration;
use ciproxy_metering::report::range::DateRange;
use ciproxy_metering::report::savings::ProviderTotals;
use ciproxy_metering::service::{Metering, StoredMetering};
use ciproxy_store_interface::jobs::{JobResult, JobState};
use ciproxy_store_interface::metering::history::OverflowCost;
use ciproxy_store_interface::metering::totals::LocationTotals;
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use ciproxy_store_interface::runners::RunnerPlace;

use super::super::support::{Database, at, job, machine, runner};
use super::support::{prices, usd};

#[test]
fn savings_keeps_provider_usage_and_idle_cost_separate_from_own_machines() {
    let database = Database::new();
    let workspace = database.workspace();
    let organization = database.organization(&workspace);
    let (label, scale) = database.label_and_scale_set(&organization);
    let own = machine();
    database.store.create_machine(workspace.id, &own).unwrap();
    let provider = OverflowProvider::Boat;
    let sandbox = |id: &str| RunnerPlace::Overflow {
        provider,
        key_id: None,
        sandbox_id: id.to_owned(),
        size_name: "2cpu".to_owned(),
    };
    for (place, github_id, minutes) in [
        (
            RunnerPlace::Machine {
                machine_id: own.id,
                first_slot: 0,
            },
            41,
            3,
        ),
        (sandbox("with-job"), 42, 5),
    ] {
        let vm = runner(&label, &scale, place);
        database.store.create_runner(workspace.id, &vm).unwrap();
        let mut taken = job(&label, &scale);
        taken.github_id = github_id;
        taken.runner_id = Some(vm.id);
        taken.started_at = Some(at());
        taken.completed_at = Some(at() + Duration::minutes(minutes));
        taken.state = JobState::Completed;
        taken.result = Some(JobResult::Success);
        database.store.insert_job(workspace.id, &taken).unwrap();
    }
    let idle = runner(&label, &scale, sandbox("without-job"));
    database.store.create_runner(workspace.id, &idle).unwrap();
    database
        .store
        .save_overflow_cost(
            workspace.id,
            &OverflowCost {
                runner_id: idle.id,
                provider,
                starts_at: at(),
                ends_at: Some(at() + Duration::minutes(10)),
                start_usd: usd("0.07"),
                minute_usd: usd("0.02"),
            },
        )
        .unwrap();

    let service = StoredMetering::new(database.store.clone());
    let observed = at() + Duration::hours(1);
    service.refresh(workspace.id, observed).unwrap();
    let range = DateRange {
        from: at().date_naive(),
        until: at().date_naive().succ_opt().unwrap(),
    };
    let report = service
        .savings(workspace.id, range, observed, &prices())
        .unwrap();

    assert_eq!(
        report.machines,
        LocationTotals {
            starts: 1,
            minutes: usd("3")
        }
    );
    assert_eq!(
        report.providers,
        [(
            provider,
            ProviderTotals {
                usage: LocationTotals {
                    starts: 1,
                    minutes: usd("5")
                },
                usd: usd("0.27"),
            }
        )]
        .into()
    );
    assert_eq!(report.days[0].sizes.len(), 1);
    assert_eq!(
        report.days[0].overflow_usd,
        [(provider, usd("0.27"))].into()
    );
}
