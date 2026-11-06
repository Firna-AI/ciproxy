//! Missing authoritative timing of one exact scale-set assignment attempt.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::github::GithubJob;

/// Raw demand remains the proof until authoritative timing is recovered.
#[derive(Clone, Debug)]
pub struct PendingTiming {
    /// Internal identity of this exact assignment-order attempt.
    pub attempt_id: Uuid,
    /// Trusted scale-set identity and API repository/run facts.
    pub job: GithubJob,
}

/// Real Actions job metadata matched by run and unique ephemeral runner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JobTimes {
    /// Actual numeric Actions identity returned by GitHub, never inferred.
    pub actions_job_id: i64,
    /// Real start, absent until GitHub supplies it.
    pub started_at: Option<DateTime<Utc>>,
    /// Real finish, absent until GitHub supplies it.
    pub completed_at: Option<DateTime<Utc>>,
    /// GitHub reports the attempt completed even if its finish is unavailable.
    pub terminal: bool,
}
