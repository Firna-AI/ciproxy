//! StepsRunner trace records in the pinned runner's Worker log.

use std::io::{self, ErrorKind};
use std::ops::Range;

use ciproxy_protocol::usage::{JobStep, JobStepTiming, MAX_STEP_NAME_CHARS, MAX_STEPS};

/// Ordered step observations, including one step with an unknown outcome.
#[derive(Default)]
pub(super) struct Steps {
    finished: Vec<JobStep>,
    current: Option<PendingStep>,
}

struct PendingStep {
    step: JobStep,
    last_at_ms: i64,
}

enum Event<'line> {
    Processing(&'line str),
    Starting,
    Result,
    Skipping,
}

impl<'line> Event<'line> {
    fn from_message(message: &'line str) -> io::Result<Option<Self>> {
        if let Some(name) = message.strip_prefix("Processing step:") {
            let name = name
                .strip_prefix(" DisplayName='")
                .and_then(|name| name.strip_suffix('\''))
                .ok_or(ErrorKind::InvalidData)?;
            Ok(Some(Self::Processing(name)))
        } else if message == "Starting the step." {
            Ok(Some(Self::Starting))
        } else if let Some(result) = message.strip_prefix("Step result:") {
            if !result.starts_with(' ') {
                return Err(ErrorKind::InvalidData.into());
            }
            Ok(Some(Self::Result))
        } else if message == "Skipping step due to condition evaluation." {
            Ok(Some(Self::Skipping))
        } else {
            Ok(None)
        }
    }
}

impl Steps {
    /// Consume one complete trace line in the pinned Worker log format.
    pub(super) fn observe(&mut self, line: &str, first_second_ms: i64) -> io::Result<()> {
        if self.full() {
            return Ok(());
        }
        let Some((header, message)) = line
            .strip_prefix('[')
            .and_then(|line| line.split_once("] "))
        else {
            return Ok(());
        };
        let Some(stamp) = header.strip_suffix(" INFO StepsRunner") else {
            return Ok(());
        };
        let Some(event) = Event::from_message(message)? else {
            return Ok(());
        };
        let at_ms = utc_millis(stamp).ok_or(ErrorKind::InvalidData)?;

        if let Event::Processing(name) = event {
            self.finish_step(first_second_ms);
            if !self.full() {
                self.current = Some(PendingStep {
                    step: JobStep {
                        name: name.chars().take(MAX_STEP_NAME_CHARS).collect(),
                        timing: JobStepTiming::Ran {
                            started_at_ms: None,
                            ended_at_ms: None,
                        },
                    },
                    last_at_ms: at_ms,
                });
            }
            return Ok(());
        }

        let current = self.current.as_mut().ok_or(ErrorKind::InvalidData)?;
        current.last_at_ms = current.last_at_ms.max(at_ms);
        match (&mut current.step.timing, event) {
            (JobStepTiming::Ran { started_at_ms, .. }, Event::Starting)
                if started_at_ms.is_none() =>
            {
                *started_at_ms = Some(at_ms);
            }
            (JobStepTiming::Ran { ended_at_ms, .. }, Event::Result) => {
                *ended_at_ms = Some(at_ms);
                self.finish_step(first_second_ms);
            }
            (
                JobStepTiming::Ran {
                    started_at_ms: None,
                    ..
                },
                Event::Skipping,
            ) => {
                current.step.timing = JobStepTiming::Skipped;
                self.finish_step(first_second_ms);
            }
            _ => return Err(ErrorKind::InvalidData.into()),
        }
        Ok(())
    }

    /// Whether the report already has its first full list of steps.
    pub(super) fn full(&self) -> bool {
        self.finished.len() == MAX_STEPS
    }

    /// Keep steps with an event in the sample window, including an open step.
    pub(super) fn snapshot(&self, first_second_ms: i64) -> Vec<JobStep> {
        let mut steps = self.finished.clone();
        if let Some(pending) = &self.current
            && pending.last_at_ms >= first_second_ms
        {
            steps.push(pending.step.clone());
        }
        steps
    }

    fn finish_step(&mut self, first_second_ms: i64) {
        if let Some(pending) = self.current.take()
            && pending.last_at_ms >= first_second_ms
        {
            self.finished.push(pending.step);
        }
    }
}

fn utc_millis(stamp: &str) -> Option<i64> {
    let bytes = stamp.as_bytes();
    if !matches!(
        bytes,
        [
            _,
            _,
            _,
            _,
            b'-',
            _,
            _,
            b'-',
            _,
            _,
            b' ',
            _,
            _,
            b':',
            _,
            _,
            b':',
            _,
            _,
            b'Z'
        ]
    ) {
        return None;
    }
    let decimal = |range: Range<usize>| {
        bytes[range].iter().try_fold(0_i64, |value, digit| {
            digit
                .is_ascii_digit()
                .then(|| value * 10 + i64::from(digit - b'0'))
        })
    };
    let year = decimal(0..4)?;
    let month = decimal(5..7)?;
    let day = decimal(8..10)?;
    let hour = decimal(11..13)?;
    let minute = decimal(14..16)?;
    let second = decimal(17..19)?;
    if year == 0 || !(1..=12).contains(&month) || hour >= 24 || minute >= 60 || second >= 60 {
        return None;
    }
    let month_starts = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365];
    let month_index = (month - 1) as usize;
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let month_days =
        month_starts[month_index + 1] - month_starts[month_index] + i64::from(leap && month == 2);
    if !(1..=month_days).contains(&day) {
        return None;
    }
    let days = (year - 1970) * 365 + leap_days_before(year) - leap_days_before(1970)
        + month_starts[month_index]
        + i64::from(leap && month > 2)
        + day
        - 1;
    Some((days * 86_400 + hour * 3_600 + minute * 60 + second) * 1_000)
}

fn leap_days_before(year: i64) -> i64 {
    let previous = year - 1;
    previous / 4 - previous / 100 + previous / 400
}

#[cfg(test)]
#[path = "_tests_/steps_tests.rs"]
mod steps_tests;
