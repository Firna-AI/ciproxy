//! Preparation precedes acknowledgement and protects an assigned job from stop.

use ciproxy_protocol::guest::JobDecision;
use std::collections::VecDeque;
use std::future::pending;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use ciproxy_protocol::guest::{AgentToGuest, GuestToAgent};
use ciproxy_protocol::identifiers::{JobId, RunnerId};
use ciproxy_protocol::runners::RunnerExit;
use ciproxy_protocol::runtime::{CachePermissions, RuntimeCredentials};
use ciproxy_protocol::secret::Secret;
use unimock::{MockFn, Unimock, matching};

use crate::boundary::{ChannelMock, RunnerEvent, RunnerMock};
use crate::service::Guest;

use super::service_tests::support::{configure, supervisor};

#[tokio::test]
async fn preparation_ack_follows_local_readiness_and_late_ready_cannot_replace_assignment() {
    for fail_preparation in [false, true] {
        let runtime = RuntimeCredentials {
            docker_data_bytes: 0,
            proxy_url: "http://172.21.0.1:8230/".to_owned(),
            github_token: Secret::new("original-runtime"),
            cache_url: Secret::new("https://cache.example/"),
            results_url: Secret::new("https://results.example/"),
            permissions: CachePermissions {
                read: true,
                write: false,
            },
        };
        let commands = Arc::new(Mutex::new(VecDeque::from(vec![
            Some(configure()),
            Some(AgentToGuest::PrepareJob {
                job_id: JobId("job-1".to_owned()),
                decision: JobDecision::GitHubOnly,
            }),
            Some(AgentToGuest::Shutdown),
            None,
        ])));
        let assigned = Arc::new(AtomicBool::new(false));
        let assigned_send = assigned.clone();
        let messages = Arc::new(Mutex::new(Vec::new()));
        let observed = messages.clone();
        let timeline = Arc::new(Mutex::new(Vec::new()));
        let acknowledged = timeline.clone();
        let prepared = timeline.clone();
        let control = Unimock::new((
            ChannelMock::receive
                .each_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    let commands = commands.clone();
                    let assigned = assigned.clone();
                    Box::pin(async move {
                        let blocked = matches!(
                            commands.lock().unwrap().front(),
                            Some(Some(AgentToGuest::PrepareJob { .. }))
                        ) && !assigned.load(Ordering::SeqCst);
                        if blocked {
                            return pending().await;
                        }
                        Ok(commands.lock().unwrap().pop_front().unwrap())
                    })
                })),
            ChannelMock::send
                .each_call(matching!(_))
                .answers_arc(Arc::new(move |_, message| {
                    if matches!(message, GuestToAgent::JobStarting { .. }) {
                        assigned_send.store(true, Ordering::SeqCst);
                    }
                    if matches!(message, GuestToAgent::JobPrepared { .. }) {
                        acknowledged.lock().unwrap().push("acknowledged");
                    }
                    observed.lock().unwrap().push(message);
                    Ok(())
                })),
        ));
        let events = Arc::new(Mutex::new(VecDeque::from(vec![
            RunnerEvent::Starting {
                job_id: JobId("job-1".to_owned()),
                runtime: runtime.clone(),
            },
            RunnerEvent::Ready,
            RunnerEvent::Exited(RunnerExit { code: Some(0) }),
        ])));
        let process = Unimock::new((
            RunnerMock::start.some_call(matching!(_)).returns(Ok(())),
            RunnerMock::next_event
                .each_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    let events = events.clone();
                    Box::pin(async move { Ok(events.lock().unwrap().pop_front().unwrap()) })
                })),
            RunnerMock::prepare
                .some_call(matching!(_, _))
                .answers_arc(Arc::new(move |_, _, _| {
                    prepared.lock().unwrap().push("prepared");
                    if fail_preparation {
                        Err(crate::error::Error::PoweroffFailed)
                    } else {
                        Ok(ciproxy_protocol::guest::JobPreparation::default())
                    }
                })),
        ));
        let trace = Arc::new(Mutex::new(Vec::new()));
        let mut guest = supervisor(control, process, &trace);
        guest.run().await.unwrap();
        assert_eq!(*timeline.lock().unwrap(), vec!["prepared", "acknowledged"]);
        let messages = messages.lock().unwrap();
        assert_eq!(messages.len(), 5);
        assert_eq!(
            messages[1],
            GuestToAgent::RunnerConfigured {
                runner_id: RunnerId("runner-7".to_owned())
            }
        );
        assert_eq!(
            messages[2],
            GuestToAgent::JobStarting {
                runner_id: RunnerId("runner-7".to_owned()),
                job_id: JobId("job-1".to_owned()),
                runtime
            }
        );
        assert_eq!(
            messages[3],
            GuestToAgent::JobPrepared {
                outcome: ciproxy_protocol::guest::JobPreparation::default(),
                runner_id: RunnerId("runner-7".to_owned()),
                job_id: JobId("job-1".to_owned())
            }
        );
        assert_eq!(
            messages[4],
            GuestToAgent::RunnerExited {
                runner_id: RunnerId("runner-7".to_owned()),
                exit: RunnerExit { code: Some(0) }
            }
        );
    }
}
