//! CPU, memory, and step observations from a job VM.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::platform::Platform;

/// Maximum number of points in a report.
pub const MAX_POINTS: usize = 3_600;
/// Maximum number of steps in a report.
pub const MAX_STEPS: usize = 1_000;
/// Maximum number of Unicode scalar values in a step name.
pub const MAX_STEP_NAME_CHARS: usize = 256;

/// Whole-VM usage with one entry in each sample array per point.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JobUsage {
    /// Platform of the VM that ran the job.
    pub platform: Platform,
    /// Start of the first point in UTC milliseconds since the Unix epoch.
    pub first_point_at_ms: i64,
    /// Width of each point in seconds. Merging points doubles this value.
    pub point_seconds: u32,
    /// Average busy CPU time in thousandths of a vCPU for each point.
    pub cpu_milli_vcpu: Vec<u32>,
    /// Peak memory in MiB for each point.
    pub peak_memory_mib: Vec<u32>,
    /// Number of out-of-memory kills for each point, absent on macOS.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oom_kills: Option<Vec<u32>>,
    /// Steps in runner order, or an empty list when unavailable.
    pub steps: Vec<JobStep>,
}

/// A named step observed in the runner log.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JobStep {
    /// The display name from the runner log.
    pub name: String,
    /// Observed times or an explicit skip.
    #[serde(flatten)]
    pub timing: JobStepTiming,
}

/// Whether the runner ran or skipped a step.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum JobStepTiming {
    /// The runner did not skip the step. Missing times mean an incomplete log.
    Ran {
        /// Start in UTC milliseconds since the Unix epoch, when observed.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        started_at_ms: Option<i64>,
        /// End in UTC milliseconds since the Unix epoch, when observed.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ended_at_ms: Option<i64>,
    },
    /// The runner skipped the step, so it has no start or end.
    Skipped,
}

/// A structural limit or requirement that a usage report does not meet.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UsageValidationError {
    /// The report has more than [`MAX_POINTS`] points.
    TooManyPoints,
    /// The sample arrays have different lengths.
    MismatchedPointArrays,
    /// The point width in seconds is zero or not a power of two.
    InvalidPointSeconds,
    /// The kill counts are missing on Linux or present on macOS.
    InvalidOomKills,
    /// The report has more than [`MAX_STEPS`] steps.
    TooManySteps,
    /// A step name exceeds [`MAX_STEP_NAME_CHARS`] characters.
    StepNameTooLong,
}

impl fmt::Display for UsageValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::TooManyPoints => "too many points",
            Self::MismatchedPointArrays => "sample arrays have different lengths",
            Self::InvalidPointSeconds => "point width is not a positive power of two",
            Self::InvalidOomKills => "kill counts do not match the platform",
            Self::TooManySteps => "too many steps",
            Self::StepNameTooLong => "step name is too long",
        };
        write!(formatter, "[protocol/usage] {message}")
    }
}

impl std::error::Error for UsageValidationError {}

impl JobUsage {
    /// Check the point and step structure without knowing the VM or runner life.
    pub fn validate(&self) -> Result<(), UsageValidationError> {
        self.validate_points()?;
        self.validate_steps()
    }

    /// Check only the point limits, array lengths, and kill count presence.
    pub fn validate_points(&self) -> Result<(), UsageValidationError> {
        let points = self.cpu_milli_vcpu.len();
        if points > MAX_POINTS {
            return Err(UsageValidationError::TooManyPoints);
        }
        if !self.point_seconds.is_power_of_two() {
            return Err(UsageValidationError::InvalidPointSeconds);
        }
        if !matches!(
            (self.platform, &self.oom_kills),
            (Platform::LinuxX64, Some(_)) | (Platform::MacosArm64, None)
        ) {
            return Err(UsageValidationError::InvalidOomKills);
        }
        if self.peak_memory_mib.len() != points
            || self
                .oom_kills
                .as_ref()
                .is_some_and(|kills| kills.len() != points)
        {
            return Err(UsageValidationError::MismatchedPointArrays);
        }
        Ok(())
    }

    /// Check only the step and name limits, so a reader can keep valid samples.
    pub fn validate_steps(&self) -> Result<(), UsageValidationError> {
        if self.steps.len() > MAX_STEPS {
            return Err(UsageValidationError::TooManySteps);
        }
        if self
            .steps
            .iter()
            .any(|step| step.name.chars().nth(MAX_STEP_NAME_CHARS).is_some())
        {
            return Err(UsageValidationError::StepNameTooLong);
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "_tests_/usage_tests.rs"]
mod usage_tests;
