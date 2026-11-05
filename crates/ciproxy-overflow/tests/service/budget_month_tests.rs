//! Cross-month totals bound gaps without inventing measured monthly costs.

use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use ciproxy_overflow::api::Overflow;
use ciproxy_overflow::provider::{BoatAccountApiMock, RuntimeUsage};
use ciproxy_store_interface::overflow::known::KnownSandbox;
use ciproxy_store_interface::overflow::record::{RuntimeCharge, StartPhase};
use rust_decimal::Decimal;
use unimock::{MockFn, Unimock, matching};

use super::harness::{Harness, Runtime};

use super::fixtures::{capabilities, id, record};

#[tokio::test]
async fn expired_cross_month_total_retains_unallocated_cost_without_a_measured_interval() {
    let since = "2026-11-30T18:00:00Z".parse().unwrap();
    let month = "2026-12-01T00:00:00Z".parse().unwrap();
    let now = month + Duration::hours(1);
    let backend = capabilities(Unimock::new(
        BoatAccountApiMock::usage
            .some_call(matching!(_, _, _))
            .answers_arc(Arc::new(move |_, _, from, until| {
                if from == Some(month) {
                    return Ok(None);
                }
                Ok(Some(RuntimeUsage {
                    since: from.unwrap_or(since),
                    until,
                    usd: if from.is_none() {
                        Decimal::from(9)
                    } else {
                        Decimal::from(2)
                    },
                    running: false,
                }))
            })),
    ));
    let harness = Harness::new(backend, (), Runtime::Read);
    *harness.now.lock().unwrap() = now;
    {
        harness.set_checked(now);
        let mut start = record(StartPhase::Running);
        start.starts_at = since;
        start.sandbox = Some(KnownSandbox::observed(
            "bx_23456789".to_owned(),
            since,
            start.dispatch.as_ref().unwrap().lifetime_seconds,
        ));
        start.dispatch.as_mut().unwrap().size.hourly_usd = Decimal::ONE;
        start.start_usd = None;
        start.runtime.push(RuntimeCharge {
            month: "2026-11-01".parse().unwrap(),
            usd: Decimal::from(2),
            until: month,
            running: false,
        });
        harness.seed(start);
    }
    let view = harness.service.view(id(1)).await.unwrap();
    assert_eq!(view.spend.confirmed_usd, Decimal::ZERO);
    assert_eq!(view.spend.pending_usd, Decimal::ZERO);
    assert_eq!(view.spend.settled_bound_usd, Decimal::from(7));
    assert_eq!(view.scaler_account(now).unwrap().spend.spent, 7_000_000);
    assert_eq!(
        view.starts[0].runtime_total.as_ref().unwrap().usd,
        Decimal::from(9)
    );
    assert_eq!(view.starts[0].runtime.len(), 1);
    assert_eq!(
        view.starts[0].runtime[0].month,
        "2026-11-01".parse().unwrap()
    );
    assert_eq!(view.starts[0].runtime[0].usd, Decimal::from(2));
    assert_eq!(
        view.starts[0].billing.unwrap().ends_at(),
        "2026-11-30T20:00:00Z".parse::<DateTime<Utc>>().unwrap()
    );
}
