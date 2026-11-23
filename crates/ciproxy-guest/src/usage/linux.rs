//! Read VM counters and identify the stock runner's worker via Linux procfs.

use std::collections::HashMap;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::PathBuf;

use ciproxy_protocol::platform::Platform;
use rustix::param::clock_ticks_per_second;

use super::{CpuReading, Reading, Source};

/// The Linux source for the sampler.
pub(crate) struct Proc {
    root: PathBuf,
    ticks_per_second: u64,
}

impl Proc {
    /// Measure the VM that runs this guest.
    pub(crate) fn new() -> Self {
        Self {
            root: PathBuf::from("/proc"),
            ticks_per_second: clock_ticks_per_second(),
        }
    }
}

impl Source for Proc {
    fn platform(&self) -> Platform {
        Platform::LinuxX64
    }

    fn read(&self) -> io::Result<Reading> {
        let at_ms = Reading::now_ms()?;
        let stat = fs::read_to_string(self.root.join("stat"))?;
        let (busy_cpu_micros, max_cpu_milli_vcpu) =
            cpu(&stat, self.ticks_per_second).ok_or(ErrorKind::InvalidData)?;
        let meminfo = fs::read_to_string(self.root.join("meminfo"))?;
        let memory_mib = memory(&meminfo).ok_or(ErrorKind::InvalidData)?;
        let vmstat = fs::read_to_string(self.root.join("vmstat"))?;
        let oom_kills = oom_kills(&vmstat).ok_or(ErrorKind::InvalidData)?;
        Ok(Reading {
            at_ms,
            cpu: CpuReading::BusyMicros(busy_cpu_micros),
            max_cpu_milli_vcpu,
            memory_mib,
            oom_kills,
        })
    }

    fn worker_present(&self, runner_pid: u32) -> io::Result<bool> {
        let mut processes = HashMap::new();
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            let Some(pid) = entry
                .file_name()
                .to_str()
                .and_then(|name| name.parse().ok())
            else {
                continue;
            };
            let Ok(stat) = fs::read(entry.path().join("stat")) else {
                continue;
            };
            if let Some(process) = Process::from_stat(&stat) {
                processes.insert(pid, process);
            }
        }
        Ok(worker_in_tree(&processes, runner_pid))
    }
}

fn cpu(stat: &str, ticks_per_second: u64) -> Option<(u64, u32)> {
    let mut lines = stat.lines();
    let mut fields = lines.next()?.split_ascii_whitespace();
    if fields.next()? != "cpu" {
        return None;
    }
    let mut busy = 0_u64;
    for index in 0..7 {
        let ticks = fields.next()?.parse::<u64>().ok()?;
        if matches!(index, 0 | 1 | 2 | 5 | 6) {
            busy = busy.checked_add(ticks)?;
        }
    }
    let micros = (u128::from(busy) * 1_000_000).checked_div(u128::from(ticks_per_second))?;
    let cpus = lines
        .take_while(|line| {
            line.split_ascii_whitespace()
                .next()
                .and_then(|field| field.strip_prefix("cpu"))
                .is_some_and(|index| index.parse::<u32>().is_ok())
        })
        .count();
    if cpus == 0 {
        return None;
    }
    Some((
        u64::try_from(micros).ok()?,
        u32::try_from(cpus).ok()?.checked_mul(1_000)?,
    ))
}

fn memory(meminfo: &str) -> Option<u32> {
    let total = counter(meminfo, "MemTotal:")?;
    let available = counter(meminfo, "MemAvailable:")?;
    u32::try_from(total.checked_sub(available)? / 1_024).ok()
}

fn oom_kills(vmstat: &str) -> Option<u64> {
    counter(vmstat, "oom_kill")
}

fn counter(text: &str, name: &str) -> Option<u64> {
    text.lines().find_map(|line| {
        let mut fields = line.split_ascii_whitespace();
        (fields.next()? == name).then_some(())?;
        fields.next()?.parse().ok()
    })
}

struct Process {
    parent: u32,
    worker: bool,
}

impl Process {
    fn from_stat(stat: &[u8]) -> Option<Self> {
        let open = stat.iter().position(|byte| *byte == b'(')?;
        let close = stat.iter().rposition(|byte| *byte == b')')?;
        let name = stat.get(open + 1..close)?;
        let mut fields = stat[close + 1..]
            .split(|byte| byte.is_ascii_whitespace())
            .filter(|field| !field.is_empty());
        let state = fields.next()?;
        let parent = std::str::from_utf8(fields.next()?).ok()?.parse().ok()?;
        Some(Self {
            parent,
            worker: name == b"Runner.Worker" && !matches!(state, b"Z" | b"X"),
        })
    }
}

fn worker_in_tree(processes: &HashMap<u32, Process>, runner_pid: u32) -> bool {
    processes
        .values()
        .filter(|process| process.worker)
        .any(|worker| {
            let mut parent = worker.parent;
            for _ in 0..processes.len() {
                if parent == runner_pid {
                    return true;
                }
                let Some(process) = processes.get(&parent) else {
                    break;
                };
                parent = process.parent;
            }
            false
        })
}

#[cfg(test)]
#[path = "_tests_/linux_tests.rs"]
mod linux_tests;
