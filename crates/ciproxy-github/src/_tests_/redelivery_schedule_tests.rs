//! The retained attempt count controls the next retry time across restarts.

use std::sync::{Arc, Mutex};

use chrono::{DateTime, TimeDelta, Utc};
use unimock::{MockFn, Unimock, matching};

use crate::redelivery::{Delivery, DeliveryApiMock, Page, RecoverDeliveries, WebhookRecovery};
use crate::runtime::RuntimeMock;

fn time(value: &str) -> DateTime<Utc> {
    value.parse().unwrap()
}

#[tokio::test]
async fn recovery_waits_for_exponential_due_time_and_keeps_retrying_at_the_hour_cap() {
    for (count, before, due) in [
        (1, "2026-11-05T00:00:59Z", "2026-11-05T00:01:00Z"),
        (2, "2026-11-05T00:01:59Z", "2026-11-05T00:02:00Z"),
        (3, "2026-11-05T00:03:59Z", "2026-11-05T00:04:00Z"),
        (4, "2026-11-05T00:07:59Z", "2026-11-05T00:08:00Z"),
        (5, "2026-11-05T00:15:59Z", "2026-11-05T00:16:00Z"),
        (6, "2026-11-05T00:31:59Z", "2026-11-05T00:32:00Z"),
        (7, "2026-11-05T00:59:59Z", "2026-11-05T01:00:00Z"),
        (90, "2026-11-05T00:59:59Z", "2026-11-05T01:00:00Z"),
    ] {
        let mut history: Vec<_> = (1..=count)
            .map(|id| Delivery {
                id,
                guid: "failed".to_owned(),
                delivered_at: time("2026-11-05T00:00:00Z")
                    - TimeDelta::seconds((count - id).try_into().unwrap()),
                status_code: Some(503),
                event: "installation".to_owned(),
            })
            .collect();
        history.push(Delivery {
            id: 100,
            guid: "failed".to_owned(),
            delivered_at: time("2026-11-05T02:00:00Z"),
            status_code: Some(503),
            event: "issues".to_owned(),
        });
        let retried = Arc::new(Mutex::new(Vec::new()));
        let observed = retried.clone();
        let api = Arc::new(Unimock::new((
            DeliveryApiMock::list
                .each_call(matching!(None))
                .answers_arc(Arc::new(move |_, _| {
                    Ok(Page {
                        deliveries: history.clone(),
                        next: None,
                    })
                })),
            DeliveryApiMock::redeliver
                .some_call(matching!(_))
                .answers_arc(Arc::new(move |_, id| {
                    observed.lock().unwrap().push(id);
                    Ok(())
                }))
                .at_least_times(0),
        )));
        let recovery = RecoverDeliveries { api };
        let now = Arc::new(Mutex::new(time(before)));
        let clock = now.clone();
        let runtime = Unimock::new((
            RuntimeMock::now
                .each_call(matching!())
                .answers_arc(Arc::new(move |_| *clock.lock().unwrap())),
            RuntimeMock::pause.each_call(matching!(_)).returns(()),
        ));
        recovery.retry_failed(&runtime).await.unwrap();
        assert_eq!(*retried.lock().unwrap(), Vec::<u64>::new(), "count {count}");
        *now.lock().unwrap() = time(due);
        recovery.retry_failed(&runtime).await.unwrap();
        assert_eq!(*retried.lock().unwrap(), vec![count], "count {count}");
    }
}
