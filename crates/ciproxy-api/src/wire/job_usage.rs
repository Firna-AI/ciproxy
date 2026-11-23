//! CPU and memory contract for the job detail page.

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

/// Usage for the exact runner of a visible job.
#[derive(Serialize, ToSchema)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum JobUsageView {
    /// The finished job has samples from its runner.
    Measured {
        /// Figures from all the stored points, before chart merging.
        job: JobUsageFigures,
        /// Actual CPU and memory of the VM that ran the job.
        vm: JobUsageVmSize,
        /// Requested bounds to mark only where the VM was larger.
        #[schema(required = true)]
        label_mark: Option<JobUsageLabelMark>,
        /// Points and their time range for the CPU and memory plots.
        chart: JobUsageChart,
        /// Steps in runner order, or an empty list when unavailable.
        steps: Vec<JobUsageStep>,
    },
    /// The job still runs, so the API gives no figures.
    Running,
    /// There are no usage figures to show for this job.
    Unavailable,
}

/// CPU, memory, and out-of-memory kills for a window of samples.
#[derive(Serialize, ToSchema)]
pub struct JobUsageFigures {
    /// Average busy CPU in thousandths of a vCPU, rounded down.
    pub average_cpu_milli_vcpu: u32,
    /// Peak memory in bytes as a decimal string.
    pub peak_memory_bytes: String,
    /// Linux kill count, including zero. Null on macOS.
    #[schema(required = true)]
    pub oom_kills: Option<u64>,
}

/// The actual bounds of the meters and plots.
#[derive(Serialize, ToSchema)]
pub struct JobUsageVmSize {
    /// Actual virtual CPUs.
    pub vcpus: u32,
    /// Actual memory bytes as a decimal string.
    pub memory_bytes: String,
}

/// The label bounds to mark on the meters and plots.
#[derive(Serialize, ToSchema)]
pub struct JobUsageLabelMark {
    /// Requested virtual CPUs, null unless smaller than the VM's count.
    #[schema(required = true)]
    pub vcpus: Option<u32>,
    /// Requested memory bytes as a decimal string, null unless smaller.
    #[schema(required = true)]
    pub memory_bytes: Option<String>,
}

/// The chart's consecutive buckets of samples.
#[derive(Serialize, ToSchema)]
pub struct JobUsageChart {
    /// UTC start of the first point.
    pub started_at: DateTime<Utc>,
    /// UTC end of the last point. It can end before a full bucket.
    pub ended_at: DateTime<Utc>,
    /// Bucket width in seconds. The last point can be shorter.
    pub point_seconds: u64,
    /// Merged points in time order.
    #[schema(max_items = 600)]
    pub points: Vec<JobUsageFigures>,
}

/// One numbered step and its usage within the sample window.
#[derive(Serialize, ToSchema)]
pub struct JobUsageStep {
    /// Step number in display order, from 1 for Set up job.
    pub number: u32,
    /// Safe stored name or the name of a synthetic step.
    pub name: String,
    /// The name is a script placeholder and can link to the step on GitHub.
    pub script: bool,
    /// The runner did not run this step.
    pub skipped: bool,
    /// UTC start, when known.
    #[schema(required = true)]
    pub started_at: Option<DateTime<Utc>>,
    /// UTC end, when known.
    #[schema(required = true)]
    pub ended_at: Option<DateTime<Utc>>,
    /// Elapsed seconds as a decimal string, null without a valid span.
    #[schema(required = true)]
    pub duration_seconds: Option<String>,
    /// Figures from the points whose middle lies in this step, if any.
    #[schema(required = true)]
    pub figures: Option<JobUsageFigures>,
}
