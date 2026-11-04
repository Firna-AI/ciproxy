//! Savings read coherence under a deterministic registration interleaving.

use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};

use ciproxy_store_interface::metering::facts::MachineRegistration;
use ciproxy_store_interface::metering::history::MachineCost;
use ciproxy_store_interface::metering::report::SavingsSnapshot;
use ciproxy_store_interface::metering::store::MeteringStoreMock;
use ciproxy_store_interface::metering::totals::{DailyTotal, LocationTotals, SizeTotal};
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use crate::report::catalog::Alternative;
use crate::report::costs::MonthlyDivisor;
use crate::service::{Metering, StoredMetering};
use crate::test_support::{at, catalog, day, free_history, range, size, usd};

struct State {
    current: SavingsSnapshot,
    next: Option<SavingsSnapshot>,
}

#[test]
fn savings_snapshot_cannot_omit_a_concurrently_registered_machine_cost() {
    let mut costs = free_history();
    costs.machines.push(MachineCost {
        id: Uuid::from_u128(2),
        machine_id: Uuid::from_u128(1),
        starts_at: at("2026-11-04T00:00:00Z"),
        ends_at: None,
        monthly_usd: usd("60"),
    });
    let state = Arc::new(Mutex::new(State {
        current: SavingsSnapshot {
            costs: free_history(),
            ..SavingsSnapshot::default()
        },
        next: Some(SavingsSnapshot {
            machines: vec![MachineRegistration {
                machine_id: Uuid::from_u128(1),
                registered_at: Some(at("2026-11-04T00:00:00Z")),
                removed_at: None,
            }],
            days: vec![DailyTotal {
                day: day("2026-11-04"),
                boat_usd: usd("0"),
                sizes: vec![SizeTotal {
                    size: size(),
                    machines: LocationTotals {
                        starts: 1,
                        minutes: usd("2"),
                    },
                    boat: LocationTotals::default(),
                }],
            }],
            costs,
        }),
    }));
    let store = Unimock::new(
        MeteringStoreMock::savings_snapshot
            .each_call(matching!(_, _, _))
            .answers_arc(Arc::new(move |_, _, _, _| {
                let mut state = state.lock().unwrap();
                let snapshot = state.current.clone();
                if let Some(next) = state.next.take() {
                    state.current = next;
                }
                Ok(snapshot)
            })),
    );
    let service: Arc<dyn Metering> = Arc::new(StoredMetering::new(Arc::new(store)));
    let prices = [
        catalog(Alternative::Blacksmith, "1"),
        catalog(Alternative::Github, "2"),
    ];
    let divisor = MonthlyDivisor::FixedDays(NonZeroU32::new(30).unwrap());
    let during_registration = service
        .savings(
            Uuid::from_u128(1),
            range("2026-11-04", "2026-11-05"),
            at("2026-11-04T12:00:00Z"),
            &prices,
            divisor,
        )
        .unwrap();
    assert_eq!(during_registration.savings[0].usd, Some(usd("0")));
    assert_eq!(during_registration.costs.missing, vec![]);
    let after_registration = service
        .savings(
            Uuid::from_u128(1),
            range("2026-11-04", "2026-11-05"),
            at("2026-11-04T12:00:00Z"),
            &prices,
            divisor,
        )
        .unwrap();
    assert_eq!(
        (
            after_registration.comparisons[0].usd,
            after_registration.costs.known_machines_usd,
            after_registration.savings[0].usd
        ),
        (Some(usd("2")), usd("2"), Some(usd("0")))
    );
    assert_eq!(after_registration.costs.missing, vec![]);
}
