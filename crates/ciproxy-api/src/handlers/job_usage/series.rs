//! Aggregation over the stored sample series and the smaller chart series.

use std::mem::replace;

use chrono::{DateTime, Utc};
use ciproxy_protocol::usage::JobUsage;

use crate::wire::job_usage::{JobUsageChart, JobUsageFigures};

const MAX_CHART_POINTS: usize = 600;
const MIB_IN_BYTES: u64 = 1_048_576;

#[derive(Clone, Copy)]
struct Point {
    count: u32,
    cpu_sum: u64,
    peak_memory_mib: u32,
    oom_kills: u64,
}

impl Point {
    fn merge(self, other: Self) -> Self {
        Self {
            count: self.count + other.count,
            cpu_sum: self.cpu_sum + other.cpu_sum,
            peak_memory_mib: self.peak_memory_mib.max(other.peak_memory_mib),
            oom_kills: self.oom_kills + other.oom_kills,
        }
    }
}

/// Equal-width points in the VM's clock, before any chart merging.
pub(super) struct Series {
    /// Start of the first stored point.
    pub(super) started_at: DateTime<Utc>,
    /// End of the sampled time range encoded by the stored points.
    pub(super) ended_at: DateTime<Utc>,
    point_seconds: u32,
    points: Vec<Point>,
    has_kills: bool,
}

impl Series {
    /// Check the stored series before deriving times or indexing parallel arrays.
    pub(super) fn from_usage(usage: &JobUsage) -> Option<Self> {
        usage.validate_points().ok()?;
        let started_at = DateTime::from_timestamp_millis(usage.first_point_at_ms)?;
        let duration_ms = usage.cpu_milli_vcpu.len() as i64 * i64::from(usage.point_seconds) * 1000;
        let ended_at =
            DateTime::from_timestamp_millis(usage.first_point_at_ms.checked_add(duration_ms)?)?;
        let kills = usage.oom_kills.as_deref();
        let points = usage
            .cpu_milli_vcpu
            .iter()
            .zip(&usage.peak_memory_mib)
            .enumerate()
            .map(|(index, (&cpu, &memory))| Point {
                count: 1,
                cpu_sum: u64::from(cpu),
                peak_memory_mib: memory,
                oom_kills: kills.map_or(0, |counts| u64::from(counts[index])),
            })
            .collect();
        Some(Self {
            started_at,
            ended_at,
            point_seconds: usage.point_seconds,
            points,
            has_kills: kills.is_some(),
        })
    }

    /// Number of points before chart merging.
    pub(super) fn len(&self) -> usize {
        self.points.len()
    }

    /// Figures of the whole job, or none if there are no points.
    pub(super) fn whole(&self) -> Option<JobUsageFigures> {
        self.figures(self.points.iter().copied())
    }

    /// Claim raw points by midpoint in the half-open span. The first step
    /// in display order wins if the runner reports overlapping spans.
    pub(super) fn step_figures(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        claimed: &mut [bool],
    ) -> Option<JobUsageFigures> {
        let start = self.midpoint_index(start);
        let end = self.midpoint_index(end);
        self.figures((start..end).filter_map(|index| {
            (!replace(&mut claimed[index], true)).then_some(self.points[index])
        }))
    }

    /// Merge neighboring points in powers of two without dropping the tail.
    pub(super) fn chart(&self) -> JobUsageChart {
        let mut group_width = 1;
        while self.points.len().div_ceil(group_width) > MAX_CHART_POINTS {
            group_width *= 2;
        }
        JobUsageChart {
            started_at: self.started_at,
            ended_at: self.ended_at,
            point_seconds: u64::from(self.point_seconds) * group_width as u64,
            points: self
                .points
                .chunks(group_width)
                .filter_map(|group| self.figures(group.iter().copied()))
                .collect(),
        }
    }

    fn midpoint_index(&self, at: DateTime<Utc>) -> usize {
        let width_ms = i64::from(self.point_seconds) * 1000;
        let offset = at.timestamp_millis() - self.started_at.timestamp_millis() - width_ms / 2;
        let index = (offset.max(0) as u64).div_ceil(width_ms as u64);
        index.min(self.points.len() as u64) as usize
    }

    fn figures(&self, points: impl Iterator<Item = Point>) -> Option<JobUsageFigures> {
        let total = points.reduce(Point::merge)?;
        Some(JobUsageFigures {
            average_cpu_milli_vcpu: (total.cpu_sum / u64::from(total.count)) as u32,
            peak_memory_bytes: (u64::from(total.peak_memory_mib) * MIB_IN_BYTES).to_string(),
            oom_kills: self.has_kills.then_some(total.oom_kills),
        })
    }
}
