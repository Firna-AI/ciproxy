//! Shared phase, fact merge, observation, and notification revision rules.

use chrono::{DateTime, Utc};
use ciproxy_store_interface::github::DemandPhase;

/// Advance the demand notification after its saved body changes.
pub(super) const NEXT_REVISION: &str = "github_jobs.revision + 1";

/// Map the listener phase to its stored ordering.
pub(super) fn phase(phase: DemandPhase) -> i16 {
    match phase {
        DemandPhase::Available => -1,
        DemandPhase::Assigned => 0,
        DemandPhase::Started => 1,
        DemandPhase::Completed => 2,
    }
}

/// A terminal observation supplies a fallback retention anchor.
pub(super) fn completion_observation(
    phase: DemandPhase,
    observed_at: DateTime<Utc>,
) -> Option<DateTime<Utc>> {
    (phase == DemandPhase::Completed).then_some(observed_at)
}

/// Merge supplied facts while keeping an existing optional fact when absent.
pub(super) fn merged_body(table: &str, incoming: &str) -> String {
    let fields = [
        "started_at",
        "completed_at",
        "runner_github_id",
        "actions_job_id",
    ]
    .map(|field| {
        format!(
            "'{field}', COALESCE(NULLIF(({incoming})->'{field}', 'null'::jsonb), {table}.body->'{field}')"
        )
    })
    .join(", ");
    format!("({incoming}) || jsonb_build_object({fields})")
}

/// Never move the phase of an existing assignment backwards.
pub(super) fn monotonic_phase(table: &str, incoming: &str) -> String {
    format!("GREATEST({table}.phase, {incoming})")
}

/// Keep the first trusted completion observation of an assignment.
pub(super) fn completed_observed_at(table: &str, incoming: &str) -> String {
    format!("COALESCE({table}.completed_observed_at, {incoming})")
}
