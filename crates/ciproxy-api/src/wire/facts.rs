//! Stored provenance and resource offers for Machines and Settings.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// One eligible machine's physical offer from the exact prepared snapshot.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct LabelMachineFit {
    /// Stable workspace machine identity.
    pub id: String,
    /// Last reported host name.
    pub hostname: String,
    /// Whole slots available to job VMs before current occupancy.
    pub slots: String,
    /// Hardware threads supplied by those slots.
    pub vcpus: String,
    /// Memory bytes supplied by those slots.
    pub memory_bytes: String,
    /// Exact prepared job-area capacity.
    pub job_disk_bytes: Option<String>,
    /// Complete VM root required by this machine's selected image.
    pub required_root_disk_bytes: Option<String>,
    /// Whole concurrent jobs, unavailable when disk or sizing is unknown.
    pub concurrent_jobs: Option<String>,
}

/// Counts from a complete installation-visible repository snapshot.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct OrganizationRepositories {
    /// All repositories visible to the installation, including public ones.
    pub visible: String,
    /// Visible private repositories.
    pub private: String,
    /// Visible private repositories permitted by the saved access policy.
    pub allowed_private: String,
    /// Database observation of the complete snapshot.
    pub observed_at: DateTime<Utc>,
}

/// Credential-free original adder with optional retained profile facts.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct MemberProfile {
    /// Stable GitHub identity.
    pub id: String,
    /// Retained current login, unavailable after identity pruning.
    pub login: Option<String>,
    /// Optional public name from the last GitHub authorization.
    pub name: Option<String>,
    /// Retained profile image URL.
    pub avatar_url: Option<String>,
}

/// Organization membership captured when a person requested access.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct MemberOrganization {
    /// Stable organization registration in this workspace.
    pub id: String,
    /// Login observed when the person made the request.
    pub login: String,
}
