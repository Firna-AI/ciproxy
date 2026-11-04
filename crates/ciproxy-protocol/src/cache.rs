//! Cache metadata and guest connection values, never cached job contents.

use serde::{Deserialize, Serialize};

use crate::identifiers::JobId;
use crate::secret::Secret;

/// Machine cache space and the counts used to calculate seven-day hits.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MachineCacheReport {
    /// Space currently occupied by cache entries.
    pub used_bytes: u64,
    /// Space allocated to the machine cache.
    pub capacity_bytes: u64,
    /// All restores handled by the machine cache during the last seven days.
    pub restores_last_7_days: u64,
    /// Those restores whose contents the machine cache served locally.
    pub hits_last_7_days: u64,
    /// Per-job cache metadata observed by the agent.
    pub jobs: Vec<JobCacheReport>,
}

/// The cache metadata for one job.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JobCacheReport {
    /// The job whose cache use this describes.
    pub job_id: JobId,
    /// The contents' source and, only when eligible, the restore counts.
    pub usage: JobCacheUsage,
}

/// Cache sources with counts only when the machine cache handled restores.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum JobCacheUsage {
    /// All restored contents came from this machine's cache.
    Machine {
        /// Entries restored by the job.
        restored_entries: u64,
        /// Total size of the restored entries.
        restored_bytes: u64,
    },
    /// The machine cache handled restores from both content stores.
    MachineAndGithub {
        /// Entries restored by the job.
        restored_entries: u64,
        /// Total size of the restored entries.
        restored_bytes: u64,
    },
    /// Direct GitHub use or fallback, for which no counts are reported.
    Github,
}

/// The guest's access to its own machine cache.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CacheConnection {
    /// Cache URL, redacted because a URL can contain credentials.
    pub url: Secret,
    /// Access token for this runner's cache requests.
    pub token: Secret,
}

/// Space occupied by the machine's persistent repository Docker disks.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DockerDiskReport {
    /// Repositories with a Docker disk on the machine.
    pub repositories: u64,
    /// Total space occupied by those disks.
    pub used_bytes: u64,
}

/// A Docker disk made available to a guest after a job starts.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DockerDiskAttachment {
    /// Firecracker drive identifier used to find the attached disk.
    pub drive_id: String,
    /// Whether changes persist or the job uses a disposable copy.
    pub access: DockerDiskAccess,
}

/// The disk mode already selected by the agent's Docker-disk policy.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DockerDiskAccess {
    /// The selected default-branch job keeps its changes.
    Persistent,
    /// The guest works on a copy that the agent discards after its job.
    DisposableCopy,
}
