//! Cancelling a supervisor wait cannot cancel the trusted socket read.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::task::Poll;

use ciproxy_protocol::guest::HANDOFF_PROTOCOL_VERSION;
use ciproxy_protocol::identifiers::JobId;
use ciproxy_protocol::runtime::{CachePermissions, RuntimeCredentials};
use ciproxy_protocol::secret::Secret;
use tokio::sync::oneshot;
use unimock::{MockFn, Unimock, matching};

use super::{Assignment, AssignmentSourceMock, Assignments, Inbox};
use crate::handoff::Handoff;

fn retried_assignment() -> Assignment {
    Assignment {
        handoff: Handoff {
            handoff_protocol_version: HANDOFF_PROTOCOL_VERSION,
            job_id: JobId("retried-job".into()),
            runtime: RuntimeCredentials {
                docker_data_bytes: 0,
                proxy_url: String::new(),
                github_token: Secret::new("fixture-token"),
                cache_url: Secret::new("https://cache.example/"),
                results_url: Secret::new("https://results.example/"),
                permissions: CachePermissions {
                    read: false,
                    write: false,
                },
            },
        },
        reply: Box::new(Unimock::new(())),
    }
}

#[tokio::test]
async fn assignment_inbox_keeps_receiving_after_the_hook_reconnects() {
    let source = Unimock::new(AssignmentSourceMock::receive.stub(|each| {
        each.call(matching!())
            .answers(&|_| Box::pin(async { Ok(retried_assignment()) }))
            .n_times(2)
            .then()
            .answers(&|_| Box::pin(std::future::pending()));
    }));
    let mut inbox = Inbox::new(Box::new(source));
    for _ in 0..2 {
        let received = inbox
            .receive()
            .await
            .map(|assignment| assignment.handoff.job_id)
            .map_err(|_| "assignment source closed");
        assert_eq!(received, Ok(JobId("retried-job".into())));
    }
}

#[tokio::test]
async fn cancelled_supervisor_wait_does_not_discard_the_original_assignment() {
    let (started, reading) = oneshot::channel();
    let (release, blocked) = oneshot::channel();
    let gates = Arc::new(Mutex::new(Some((started, blocked))));
    let reads = Arc::new(AtomicUsize::new(0));
    let counted = reads.clone();
    let source = Unimock::new(AssignmentSourceMock::receive.stub(|each| {
        each.call(matching!())
            .answers_arc(Arc::new(move |_| {
                let (started, blocked) = gates.lock().unwrap().take().unwrap();
                let counted = counted.clone();
                Box::pin(async move {
                    counted.fetch_add(1, Ordering::SeqCst);
                    started.send(()).unwrap();
                    blocked.await.unwrap();
                    Ok(Assignment {
                        handoff: Handoff {
                            handoff_protocol_version: HANDOFF_PROTOCOL_VERSION,
                            job_id: JobId("job-1".to_owned()),
                            runtime: RuntimeCredentials {
                                docker_data_bytes: 0,
                                proxy_url: String::new(),
                                github_token: Secret::new("original-runtime"),
                                cache_url: Secret::new("https://cache.example/"),
                                results_url: Secret::new("https://results.example/"),
                                permissions: CachePermissions {
                                    read: true,
                                    write: false,
                                },
                            },
                        },
                        reply: Box::new(Unimock::new(())),
                    })
                })
            }))
            .once()
            .then()
            .answers(&|_| Box::pin(std::future::pending()));
    }));
    let mut inbox = Inbox::new(Box::new(source));
    reading.await.unwrap();
    {
        let mut waiting = inbox.receive();
        assert!(
            std::future::poll_fn(|context| Poll::Ready(waiting.as_mut().poll(context)))
                .await
                .is_pending()
        );
    }
    release.send(()).unwrap();
    let assigned = inbox
        .receive()
        .await
        .map(|assignment| assignment.handoff.job_id)
        .map_err(|_| "assignment lost");
    assert_eq!(assigned, Ok(JobId("job-1".to_owned())));
    assert_eq!(reads.load(Ordering::SeqCst), 1);
}
