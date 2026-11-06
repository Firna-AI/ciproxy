//! Recovery uses latest attempts across pages and survives an individual error.

use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use super::{Delivery, DeliveryApiMock, Page, RecoverDeliveries, WebhookRecovery};
use crate::error::Error;

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
                    ],
                    next: None,
                })
            });
        }),
        DeliveryApiMock::redeliver
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, id| {
                observed.lock().unwrap().push(id);
                if id == 4 {
                    Err(Error::AccessUnavailable(503))
                } else {
                    Ok(())
                }
            })),
    )));
    let recovery = RecoverDeliveries { api };
    recovery.retry_failed().await.unwrap();
    assert_eq!(*retried.lock().unwrap(), vec![4, 6]);
}
