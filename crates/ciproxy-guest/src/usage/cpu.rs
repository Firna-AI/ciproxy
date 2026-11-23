//! Measure a CPU rate from two reads of cumulative counters.

use std::time::Duration;

/// Cumulative CPU counters in the units that a source provides.
#[derive(Clone, Copy)]
pub(crate) enum CpuReading {
    /// Total busy time in microseconds.
    #[cfg(target_os = "linux")]
    BusyMicros(u64),
    /// Host tick counters, each modulo 2^32.
    #[cfg(any(target_os = "macos", test))]
    HostTicks {
        /// User, nice, and system ticks.
        busy: [u32; 3],
        /// Idle ticks.
        idle: u32,
    },
}

impl CpuReading {
    /// Calculate busy CPU in thousandths of a vCPU, rounded down.
    pub(super) fn since(self, previous: Self, elapsed: Duration, capacity: u32) -> Option<u32> {
        let elapsed_micros = elapsed.as_micros();
        if elapsed_micros == 0 {
            return None;
        }
        let cpu = match (self, previous) {
            #[cfg(target_os = "linux")]
            (Self::BusyMicros(next), Self::BusyMicros(previous)) => {
                u128::from(next.checked_sub(previous)?) * 1_000 / elapsed_micros
            }
            #[cfg(any(target_os = "macos", test))]
            (
                Self::HostTicks { busy, idle },
                Self::HostTicks {
                    busy: previous_busy,
                    idle: previous_idle,
                },
            ) => {
                let busy = busy
                    .into_iter()
                    .zip(previous_busy)
                    .map(|(next, previous)| u128::from(next.wrapping_sub(previous)))
                    .sum::<u128>();
                let idle = u128::from(idle.wrapping_sub(previous_idle));
                (busy * u128::from(capacity)).checked_div(busy + idle)?
            }
            #[cfg(all(test, target_os = "linux"))]
            _ => return None,
        };
        Some(cpu.min(u128::from(capacity)) as u32)
    }
}
