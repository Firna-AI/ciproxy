//! Credential-free provenance for organization and member page facts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Organization membership observed at the initial access request.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MemberOrganization {
    /// Exact registration within the requested workspace.
    pub id: Uuid,
    /// Organization login observed at request time.
    pub login: String,
}

/// Stored organization observations, separate from current installation state.
#[derive(Clone, Debug)]
pub struct OrganizationFacts {
    /// First registration observation, unavailable for legacy registrations.
    pub joined_at: Option<DateTime<Utc>>,
    /// Last complete installation-visible repository snapshot.
    pub repositories_observed_at: Option<DateTime<Utc>>,
}

/// Public identity facts retained while a role or access request exists.
#[derive(Clone, Debug)]
pub struct MemberProfile {
    /// Current GitHub login from a successful authorization.
    pub login: String,
    /// Optional public name from the last authorization.
    pub name: Option<String>,
    /// Profile image returned by GitHub.
    pub avatar_url: String,
}
