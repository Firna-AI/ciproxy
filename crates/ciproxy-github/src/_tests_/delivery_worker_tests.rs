//! A provider throttle stops the current batch and suppresses later scans.

use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use unimock::{MockFn, Unimock, matching};

use crate::error::Error;
use crate::redelivery::{Delivery, DeliveryApiMock, Page, RecoverDeliveries};
use crate::runtime::RuntimeMock;

use super::{RecoveryPass, RecoveryWorker};

fn time(value: &str) -> DateTime<Utc> {
    value.parse().unwrap()
}

#[derive(Clone, Copy, Debug)]
enum Source {
    List,
    Post,
}

#[tokio::test]
async fn recovery_worker_stops_throttled_batches_and_skips_passes_until_provider_deadline() {
    for source in [Source::List, Source::Post] {
        for (retry_after_seconds, reset_at, before, due) in [
            (
                Some(125),
                Some(time("2026-11-05T00:20:00Z")),
                "2026-11-05T00:03:04Z",
                "2026-11-05T00:03:05Z",
            ),
            (
                None,
                Some(time("2026-11-05T00:05:00Z")),
                "2026-11-05T00:04:59Z",
                "2026-11-05T00:05:00Z",
            ),
            (None, None, "2026-11-05T00:01:59Z", "2026-11-05T00:02:00Z"),
        ] {
            let now = Arc::new(Mutex::new(time("2026-11-05T00:01:00Z")));
            let clock = now.clone();
            let reads = Arc::new(Mutex::new(0));
            let listed = reads.clone();
            let posts = Arc::new(Mutex::new(Vec::new()));
            let posted = posts.clone();
            let clock = RuntimeMock::now
                .each_call(matching!())
                .answers_arc(Arc::new(move |_| *clock.lock().unwrap()));
            let runtime = Arc::new(if matches!(source, Source::Post) {
                Unimock::new((
                    clock,
                    RuntimeMock::pause.each_call(matching!(_)).returns(()),
                ))
            } else {
                Unimock::new(clock)
            });
            let list = DeliveryApiMock::list
                .each_call(matching!(None))
                .answers_arc(Arc::new(move |_, _| {
                    let mut reads = listed.lock().unwrap();
                    *reads += 1;
                    if *reads > 1 {
                        return Ok(Page {
                            deliveries: vec![],
                            next: None,
                        });
                    }
                    if matches!(source, Source::List) {
                        return Err(Error::DeliveryThrottled {
                            retry_after_seconds,
                            reset_at,
                        });
                    }
                    Ok(Page {
                        deliveries: (1..=3)
                            .map(|id| Delivery {
                                id,
                                guid: format!("failed-{id}"),
                                delivered_at: time("2026-11-05T00:00:00Z"),
                                status_code: Some(503),
                                event: "repository".to_owned(),
                            })
                            .collect(),
                        next: None,
                    })
                }));
            let api = Arc::new(if matches!(source, Source::Post) {
                Unimock::new((
                    list,
                    DeliveryApiMock::redeliver
                        .each_call(matching!(_))
                        .answers_arc(Arc::new(move |_, id| {
                            posted.lock().unwrap().push(id);
                            if id == 2 {
                                Err(Error::DeliveryThrottled {
                                    retry_after_seconds,
                                    reset_at,
                                })
                            } else {
                                Ok(())
                            }
                        })),
                ))
            } else {
                Unimock::new(list)
            });
            let worker = RecoveryWorker::new(Arc::new(RecoverDeliveries { api }), runtime);
            assert!(matches!(worker.retry_failed().await, Ok(())));
            let expected_posts = match source {
                Source::List => vec![],
                Source::Post => vec![1, 2],
            };
            assert_eq!(*posts.lock().unwrap(), expected_posts, "{source:?}");
            *now.lock().unwrap() = time(before);
            assert!(matches!(worker.retry_failed().await, Ok(())));
            assert_eq!(*reads.lock().unwrap(), 1, "{source:?}");
            assert_eq!(*posts.lock().unwrap(), expected_posts, "{source:?}");
            *now.lock().unwrap() = time(due);
            assert!(matches!(worker.retry_failed().await, Ok(())));
            assert_eq!(*reads.lock().unwrap(), 2, "{source:?}");
            assert_eq!(*posts.lock().unwrap(), expected_posts, "{source:?}");
        }
    }
}
