//! Additive readers preserve existing page deserialization with shared job types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer};

use super::job_fit::BoatLabelFit;
use super::jobs::{
    JobCacheCounts, JobCacheSource, JobLabel, JobLocation, JobStatus, JobView, JobVmSize,
};

macro_rules! reader {
    ($value:ty, $definition:ty) => {
        impl<'de> Deserialize<'de> for $value {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                <$definition>::deserialize(deserializer)
            }
        }
    };
}

#[derive(Deserialize)]
#[serde(remote = "JobStatus", rename_all = "snake_case")]
enum Status {
    Queued,
    Running,
    Completed,
}
reader!(JobStatus, Status);

#[derive(Deserialize)]
#[serde(remote = "JobLocation", tag = "kind", rename_all = "snake_case")]
enum Location {
    Machine {
        id: String,
        name: Option<String>,
        first_slot: Option<i32>,
    },
    Boat {
        size: String,
    },
}
reader!(JobLocation, Location);

#[derive(Deserialize)]
#[serde(remote = "JobVmSize")]
struct VmSize {
    vcpus: u32,
    memory_bytes: String,
    disk_bytes: String,
}
reader!(JobVmSize, VmSize);

#[derive(Deserialize)]
#[serde(remote = "JobLabel")]
struct Label {
    name: String,
    vcpus: Option<i32>,
    memory_gb: Option<i32>,
    disk_gb: Option<i32>,
}
reader!(JobLabel, Label);

#[derive(Deserialize)]
#[serde(remote = "JobCacheSource", rename_all = "snake_case")]
enum CacheSource {
    Machine,
    Github,
    MachineAndGithub,
}
reader!(JobCacheSource, CacheSource);

#[derive(Deserialize)]
#[serde(remote = "JobCacheCounts")]
struct CacheCounts {
    restored_entries: String,
    restored_bytes: String,
}
reader!(JobCacheCounts, CacheCounts);

#[derive(Deserialize)]
#[serde(remote = "JobView")]
struct Job {
    id: String,
    organization_id: String,
    organization: String,
    name: String,
    workflow: String,
    repository: String,
    git_ref: Option<String>,
    head_branch: Option<String>,
    status: JobStatus,
    result: Option<String>,
    queued_at: Option<DateTime<Utc>>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    counts_as_start: bool,
    waited_seconds: Option<String>,
    runtime_seconds: Option<String>,
    label: JobLabel,
    location: Option<JobLocation>,
    actual_size: Option<JobVmSize>,
    routing_reason: Option<String>,
    github_url: Option<String>,
    cache_source: Option<JobCacheSource>,
    cache_counts: Option<JobCacheCounts>,
}
reader!(JobView, Job);

#[derive(Deserialize)]
#[serde(remote = "BoatLabelFit", tag = "state", rename_all = "snake_case")]
enum BoatFit {
    Unavailable,
    Disabled,
    NoSizeFits,
    Fitting {
        size_name: String,
        size: JobVmSize,
        refused: bool,
    },
    LargestFallback {
        size_name: String,
        size: JobVmSize,
        refused: bool,
    },
}
reader!(BoatLabelFit, BoatFit);
