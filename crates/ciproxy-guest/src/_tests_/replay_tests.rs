//! Complete local readiness survives a lost guest observation write.

use ciproxy_protocol::guest::{GuestToAgent, JobPreparation};
use ciproxy_protocol::identifiers::{JobId, RunnerId};
use ciproxy_protocol::runtime::{CachePermissions, RuntimeCredentials};
use ciproxy_protocol::secret::Secret;

use crate::replay::Observations;

#[test]
fn reconnect_replays_assignment_then_completed_preparation() {
    let starting = GuestToAgent::JobStarting {
        runner_id: RunnerId("runner".into()),
        job_id: JobId("job".into()),
        runtime: RuntimeCredentials {
            docker_data_bytes: 5,
            proxy_url: "http://bridge:8230".into(),
            github_token: Secret::new("fixture-token"),
            cache_url: Secret::new("https://cache.example"),
            results_url: Secret::new("https://results.example"),
            permissions: CachePermissions {
                read: true,
                write: false,
            },
        },
    };
    let prepared = GuestToAgent::JobPrepared {
        runner_id: RunnerId("runner".into()),
        job_id: JobId("job".into()),
        outcome: JobPreparation {
            cache_ready: true,
            docker_attached: true,
        },
    };
    let mut observations = Observations::default();
    observations.record(&starting);
    observations.record(&prepared);
    observations.record(&GuestToAgent::RunnerReady {
        runner_id: RunnerId("runner".into()),
    });
    assert_eq!(observations.replay(), vec![starting, prepared]);
}
