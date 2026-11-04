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
    /// How restores were handled, with measured counts when available.
    pub usage: JobCacheUsage,
}

/// Restore handling with counts for every job served by the machine cache.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "handling", rename_all = "snake_case")]
pub enum JobCacheUsage {
    /// The machine cache handled restores, including any fetches from GitHub.
    MachineHandled(
        /// Measured content origins and the total restored bytes.
        CacheRestoreCounts,
    ),
    /// Direct GitHub use or fallback, for which no counts are reported.
    Github,
}

/// Measured restores handled by the machine cache, including local misses.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CacheRestoreCounts {
    /// Entries whose contents came from this machine's cache.
    pub local_entries: u64,
    /// Entries whose contents the machine cache fetched from GitHub.
    pub github_entries: u64,
    /// Total size of restored entries from both content stores.
    pub restored_bytes: u64,
}

impl CacheRestoreCounts {
    /// Derive the content source, or none when no entry was restored.
    pub fn source(self) -> Option<CacheSource> {
        match (self.local_entries > 0, self.github_entries > 0) {
            (true, false) => Some(CacheSource::Machine),
            (false, true) => Some(CacheSource::Github),
            (true, true) => Some(CacheSource::MachineAndGithub),
            (false, false) => None,
        }
    }
}

/// Content origins derived from observed restore entry counts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CacheSource {
    /// All restored contents came from this machine's cache.
    Machine,
    /// All restored contents came from the GitHub cache.
    Github,
    /// Restored contents came from both cache stores.
    MachineAndGithub,
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

#[cfg(test)]
#[path = "_tests_/cache_tests.rs"]
mod cache_tests;
