//! Turn successive counter reads into one contiguous usage series.

use std::io::{self, ErrorKind};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::usage::JobUsage;
use tokio::time::Instant;
use tracing::warn;

use super::series::{Sample, Series};
use super::{CpuReading, Source};

/// A read of the VM clock, counters, and memory.
#[derive(Clone, Copy)]
pub struct Reading {
    /// The UTC time of this read, in whole milliseconds.
    pub(crate) at_ms: i64,
    /// CPU counters since boot.
    pub(crate) cpu: CpuReading,
    /// Online CPU capacity, in thousandths of a vCPU.
    pub(crate) max_cpu_milli_vcpu: u32,
    /// Current used memory in MiB.
    pub(crate) memory_mib: u32,
    /// Cumulative out-of-memory kills, or zero if not measured.
    pub(crate) oom_kills: u64,
}

impl Reading {
    /// Read the VM wall clock in UTC milliseconds.
    pub(super) fn now_ms() -> io::Result<i64> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|elapsed| i64::try_from(elapsed.as_millis()).ok())
            .ok_or_else(|| ErrorKind::InvalidData.into())
    }

    fn since(self, previous: Self, elapsed: Duration) -> Option<Sample> {
        let cpu = self
            .cpu
            .since(previous.cpu, elapsed, self.max_cpu_milli_vcpu)?;
        let kills = self.oom_kills.checked_sub(previous.oom_kills)?;
        Some(Sample {
            cpu_milli_vcpu: cpu,
            memory_mib: self.memory_mib,
            oom_kills: u32::try_from(kills).ok()?,
        })
    }
}

/// A sampled job window. After a gap, retain the longest contiguous segment.
pub(super) struct Window {
    platform: Platform,
    started_at_ms: Option<i64>,
    previous: Option<(Instant, Reading)>,
    current: Option<Series>,
    longest: Option<Series>,
}

impl Window {
    /// Start an empty window for the source's report platform.
    pub(super) fn new(platform: Platform) -> Self {
        Self {
            platform,
            started_at_ms: None,
            previous: None,
            current: None,
            longest: None,
        }
    }

    /// Take a baseline or measure the next second. A bad read breaks the segment.
    pub(super) fn capture(&mut self, source: &dyn Source) {
        let now = Instant::now();
        let Ok(next) = source.read() else {
            warn!("job usage sample unavailable");
            self.previous = None;
            self.end_segment();
            return;
        };
        self.started_at_ms.get_or_insert(next.at_ms);
        let Some((taken, previous)) = self.previous.replace((now, next)) else {
            return;
        };
        let elapsed = now - taken;
        if elapsed < Duration::from_secs(2)
            && let Some(sample) = next.since(previous, elapsed)
        {
            self.current
                .get_or_insert_with(|| Series::new(previous.at_ms))
                .push(sample);
        } else {
            warn!("job usage sample window unavailable");
            self.end_segment();
        }
    }

    /// Time of the first baseline, even if a later gap breaks the series.
    pub(super) fn started_at_ms(&self) -> Option<i64> {
        self.started_at_ms
    }

    /// Report the longest segment without shifting points across unread seconds.
    pub(super) fn report(&self) -> Option<JobUsage> {
        self.segment().map(|series| series.report(self.platform))
    }

    fn segment(&self) -> Option<&Series> {
        [self.longest.as_ref(), self.current.as_ref()]
            .into_iter()
            .flatten()
            .max_by_key(|series| series.seconds())
    }

    fn end_segment(&mut self) {
        if let Some(current) = self.current.take()
            && self
                .longest
                .as_ref()
                .is_none_or(|longest| current.seconds() >= longest.seconds())
        {
            self.longest = Some(current);
        }
    }
}
