//! Recovery uses latest attempts across pages and survives an individual error.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::DateTime;
use unimock::{MockFn, Unimock, matching};

use super::{Delivery, DeliveryApiMock, Page, RecoverDeliveries, WebhookRecovery};
use crate::error::Error;
use crate::runtime::RuntimeMock;

#[derive(Clone, Debug, Eq, PartialEq)]
enum Operation {
    Post(u64),
    Pause(Duration),
}

fn delivery(id: u64, guid: &str, hour: u32, status: Option<u16>, event: &str) -> Delivery {
    Delivery {
        id,
        guid: guid.to_owned(),
        delivered_at: format!("2026-11-05T{hour:02}:00:00Z").parse().unwrap(),
        status_code: status,
        event: event.to_owned(),
    }
}

#[tokio::test]
async fn recovery_retries_latest_failures_across_pages_and_preserves_successful_redeliveries() {
    let retried = Arc::new(Mutex::new(Vec::new()));
    let observed = retried.clone();
    let operations = Arc::new(Mutex::new(Vec::new()));
    let posts = operations.clone();
    let pauses = operations.clone();
    let runtime = Unimock::new((
        RuntimeMock::now.each_call(matching!()).returns(
            DateTime::parse_from_rfc3339("2026-11-05T04:00:00Z")
                .unwrap()
                .to_utc(),
        ),
        RuntimeMock::pause
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, duration| {
                pauses.lock().unwrap().push(Operation::Pause(duration));
            })),
    ));
    let api = Arc::new(Unimock::new((
        DeliveryApiMock::list.stub(|each| {
            each.call(matching!(None)).answers(&|_, _| {
                Ok(Page {
                    deliveries: vec![
                        delivery(1, "successful-later", 1, Some(503), "installation"),
                        delivery(3, "still-failing", 1, Some(503), "repository"),
                    ],
                    next: Some("next".to_owned()),
                })
            });
            each.call(matching!(Some(_))).answers(&|_, _| {
                Ok(Page {
                    deliveries: vec![
                        delivery(2, "successful-later", 2, Some(204), "installation"),
                        delivery(4, "still-failing", 2, None, "repository"),
                        delivery(5, "ignored", 3, Some(500), "issues"),
                        delivery(6, "another", 3, Some(500), "installation_repositories"),
                        delivery(7, "redirect-before-route", 3, Some(302), "installation"),
                        delivery(8, "proxy-page-before-route", 3, Some(200), "repository"),
                    ],
                    next: None,
                })
            });
        }),
        DeliveryApiMock::redeliver
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, id| {
                observed.lock().unwrap().push(id);
                posts.lock().unwrap().push(Operation::Post(id));
                if id == 4 {
                    Err(Error::AccessUnavailable(503))
                } else {
                    Ok(())
                }
            })),
    )));
    let recovery = RecoverDeliveries { api };
    recovery.retry_failed(&runtime).await.unwrap();
    assert_eq!(*retried.lock().unwrap(), vec![4, 6, 7, 8]);
    let expected = vec![
        Operation::Pause(Duration::from_secs(1)),
        Operation::Post(4),
        Operation::Pause(Duration::from_secs(1)),
        Operation::Post(6),
        Operation::Pause(Duration::from_secs(1)),
        Operation::Post(7),
        Operation::Pause(Duration::from_secs(1)),
        Operation::Post(8),
    ];
    assert_eq!(*operations.lock().unwrap(), expected);
    recovery.retry_failed(&runtime).await.unwrap();
    assert_eq!(*retried.lock().unwrap(), vec![4, 6, 7, 8, 4, 6, 7, 8]);
    assert_eq!(
        *operations.lock().unwrap(),
        [expected.clone(), expected].concat()
    );
}
