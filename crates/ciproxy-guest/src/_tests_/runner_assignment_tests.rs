//! A cached host decision releases retries without repeating preparation.

use ciproxy_protocol::guest::{HANDOFF_PROTOCOL_VERSION, JobDecision};
use ciproxy_protocol::identifiers::JobId;
use ciproxy_protocol::runtime::{CachePermissions, RuntimeCredentials};
use ciproxy_protocol::secret::Secret;
use unimock::{MockFn, Unimock, matching};

use crate::assignment::{Assignment, WorkerReplyMock};
use crate::handoff::Handoff;
use crate::runner_assignment::{AssignedJob, WaitingJob};

fn assignment(token: &str) -> Assignment {
    Assignment {
        handoff: Handoff {
            handoff_protocol_version: HANDOFF_PROTOCOL_VERSION,
            job_id: JobId("job-1".into()),
            runtime: RuntimeCredentials {
                docker_data_bytes: 0,
                proxy_url: String::new(),
                github_token: Secret::new(token),
                cache_url: Secret::new("https://cache.example/"),
                results_url: Secret::new("https://results.example/"),
                permissions: CachePermissions {
                    read: true,
                    write: false,
                },
            },
        },
        reply: Box::new(Unimock::new(
            WorkerReplyMock::ready
                .some_call(matching!(None))
                .answers(&|_, _| Ok(())),
        )),
    }
}

#[tokio::test]
async fn github_only_decision_replies_to_reconnected_hook_with_original_handoff() {
    let mut job = WaitingJob::new(assignment("original-token"));
    let first = job.decide(JobDecision::GitHubOnly).await.unwrap();
    assert!(!first.cache_ready);
    assert!(job.reconnect(assignment("replacement-token")));
    let repeated = job.decide(JobDecision::GitHubOnly).await.unwrap();
    assert_eq!(repeated, first);
    assert_eq!(
        job.handoff().runtime.github_token,
        Secret::new("original-token")
    );
}
