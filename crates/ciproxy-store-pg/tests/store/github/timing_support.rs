//! Exact assignment reads and ordered listener/timing write overlap.

use std::thread;

use chrono::{DateTime, Duration, Utc};
use ciproxy_schema::github::github_jobs;
use ciproxy_schema::github_attempts::github_job_attempts;
use ciproxy_store_interface::github::GithubJob;
use ciproxy_store_interface::github_timing::JobTimes;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::store::metering::concurrent::support::{backend_pid, tagged_store, wait_for_blocker};
use crate::store::support::{Database, at};

/// Read the exact retained assignment and its first completion observation.
pub(super) fn saved_attempt(
    database: &mut Database,
    workspace_id: Uuid,
    attempt_id: Uuid,
) -> (GithubJob, Option<DateTime<Utc>>) {
    let (body, observed) = github_job_attempts::table
        .filter(github_job_attempts::workspace_id.eq(workspace_id))
        .find(attempt_id)
        .select((
            github_job_attempts::body,
            github_job_attempts::completed_observed_at,
        ))
        .first::<(serde_json::Value, Option<DateTime<Utc>>)>(&mut database.connection)
        .unwrap();
    (serde_json::from_value(body).unwrap(), observed)
}

/// Read the current demand's independent completion observation.
pub(super) fn saved_completion_observation(
    database: &mut Database,
    workspace_id: Uuid,
    job: &GithubJob,
) -> Option<DateTime<Utc>> {
    github_jobs::table
        .filter(github_jobs::workspace_id.eq(workspace_id))
        .filter(github_jobs::organization_id.eq(job.organization_id))
        .filter(github_jobs::job_id.eq(job.id))
        .select(github_jobs::completed_observed_at)
        .first(&mut database.connection)
        .unwrap()
}

/// Queue timing behind a listener while both wait on the current demand.
pub(super) fn timing_after_listener(
    database: &mut Database,
    workspace_id: Uuid,
    attempt_id: Uuid,
    completion: GithubJob,
    times: JobTimes,
) {
    let listener = tagged_store(&database.url);
    let timing = tagged_store(&database.url);
    let mut observer = PgConnection::establish(&database.url).unwrap();
    let controller = backend_pid(&mut database.connection);
    database.connection.batch_execute("BEGIN").unwrap();
    github_jobs::table
        .filter(github_jobs::workspace_id.eq(workspace_id))
        .filter(github_jobs::organization_id.eq(completion.organization_id))
        .filter(github_jobs::job_id.eq(completion.id))
        .select(github_jobs::job_id)
        .for_update()
        .first::<Uuid>(&mut database.connection)
        .unwrap();
    let listener_pid = listener.pid;
    let listener_write = thread::spawn(move || {
        listener
            .store
            .save_github_job(workspace_id, &completion, at())
    });
    wait_for_blocker(&mut observer, listener_pid, controller);
    let timing_pid = timing.pid;
    let timing_write = thread::spawn(move || {
        timing.store.save_github_job_times(
            workspace_id,
            attempt_id,
            &times,
            at() + Duration::minutes(1),
        )
    });
    wait_for_blocker(&mut observer, timing_pid, listener_pid);
    database.connection.batch_execute("COMMIT").unwrap();
    listener_write.join().unwrap().unwrap();
    timing_write.join().unwrap().unwrap();
}
