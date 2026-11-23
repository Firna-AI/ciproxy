//! Fixed-width sample points with bounded storage.

use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::usage::{JobUsage, MAX_POINTS};

/// Usage observed during one second.
#[derive(Clone, Copy)]
pub(super) struct Sample {
    pub(super) cpu_milli_vcpu: u32,
    pub(super) memory_mib: u32,
    pub(super) oom_kills: u32,
}

#[derive(Clone, Copy)]
struct Point {
    cpu_sum: u64,
    peak_memory_mib: u32,
    oom_kills: u32,
}

impl Point {
    fn merge(self, next: Self) -> Self {
        Self {
            cpu_sum: self.cpu_sum + next.cpu_sum,
            peak_memory_mib: self.peak_memory_mib.max(next.peak_memory_mib),
            oom_kills: self.oom_kills.saturating_add(next.oom_kills),
        }
    }
}

/// A contiguous series whose last point can still be accumulating samples.
pub(super) struct Series {
    first_point_at_ms: i64,
    point_seconds: u32,
    points: Vec<Point>,
    last_seconds: u32,
}

impl Series {
    /// Set the start time of the first one-second window.
    pub(super) fn new(first_point_at_ms: i64) -> Self {
        Self {
            first_point_at_ms,
            point_seconds: 1,
            points: Vec::new(),
            last_seconds: 0,
        }
    }

    /// Add the next second, making room without losing any existing sample.
    pub(super) fn push(&mut self, sample: Sample) {
        let point = Point {
            cpu_sum: u64::from(sample.cpu_milli_vcpu),
            peak_memory_mib: sample.memory_mib,
            oom_kills: sample.oom_kills,
        };
        if self.last_seconds < self.point_seconds
            && let Some(last) = self.points.last_mut()
        {
            *last = last.merge(point);
            self.last_seconds += 1;
            return;
        }
        if self.points.len() == MAX_POINTS {
            let Some(doubled) = self.point_seconds.checked_mul(2) else {
                return;
            };
            let pairs = self.points.len() / 2;
            for index in 0..pairs {
                self.points[index] = self.points[index * 2].merge(self.points[index * 2 + 1]);
            }
            self.points.truncate(pairs);
            self.point_seconds = doubled;
        }
        self.points.push(point);
        self.last_seconds = 1;
    }

    /// Number of sampled seconds, including the filled part of the last point.
    pub(super) fn seconds(&self) -> u64 {
        self.points.len().saturating_sub(1) as u64 * u64::from(self.point_seconds)
            + u64::from(self.last_seconds)
    }

    /// Encode the points with CPU averages rounded down, without any steps.
    pub(super) fn report(&self, platform: Platform) -> JobUsage {
        let mut report = JobUsage {
            platform,
            first_point_at_ms: self.first_point_at_ms,
            point_seconds: self.point_seconds,
            cpu_milli_vcpu: Vec::with_capacity(self.points.len()),
            peak_memory_mib: Vec::with_capacity(self.points.len()),
            oom_kills: (platform == Platform::LinuxX64)
                .then(|| Vec::with_capacity(self.points.len())),
            steps: Vec::new(),
        };
        for (index, point) in self.points.iter().enumerate() {
            let seconds = if index + 1 == self.points.len() {
                self.last_seconds
            } else {
                self.point_seconds
            };
            report
                .cpu_milli_vcpu
                .push((point.cpu_sum / u64::from(seconds)) as u32);
            report.peak_memory_mib.push(point.peak_memory_mib);
            if let Some(kills) = &mut report.oom_kills {
                kills.push(point.oom_kills);
            }
        }
        report
    }
}

#[cfg(test)]
#[path = "_tests_/series_tests.rs"]
mod series_tests;
