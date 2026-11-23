//! Structural checks for samples and steps in a usage report.

use crate::platform::Platform;
use crate::wire_tests::fixtures::job_usage;

use super::UsageValidationError;

#[test]
fn rejects_more_than_3600_points() {
    let mut usage = job_usage();
    usage.cpu_milli_vcpu = vec![0; 3_600];
    usage.peak_memory_mib = vec![0; 3_600];
    usage.oom_kills = Some(vec![0; 3_600]);
    assert_eq!(usage.validate(), Ok(()));

    usage.cpu_milli_vcpu.push(0);
    usage.peak_memory_mib.push(0);
    usage.oom_kills.as_mut().unwrap().push(0);
    assert_eq!(usage.validate(), Err(UsageValidationError::TooManyPoints));
}

#[test]
fn rejects_unequal_sample_array_lengths() {
    let mut usage = job_usage();
    usage.peak_memory_mib.pop();
    assert_eq!(
        usage.validate(),
        Err(UsageValidationError::MismatchedPointArrays)
    );

    let mut usage = job_usage();
    usage.oom_kills.as_mut().unwrap().push(0);
    assert_eq!(
        usage.validate(),
        Err(UsageValidationError::MismatchedPointArrays)
    );
}

#[test]
fn accepts_only_positive_power_of_two_point_widths() {
    let mut usage = job_usage();
    for (seconds, expected) in [
        (1, Ok(())),
        (2, Ok(())),
        (8, Ok(())),
        (0, Err(UsageValidationError::InvalidPointSeconds)),
        (3, Err(UsageValidationError::InvalidPointSeconds)),
    ] {
        usage.point_seconds = seconds;
        assert_eq!(usage.validate(), expected, "point width: {seconds}");
    }
}

#[test]
fn requires_kill_counts_only_on_linux() {
    let mut usage = job_usage();
    usage.platform = Platform::MacosArm64;
    assert_eq!(usage.validate(), Err(UsageValidationError::InvalidOomKills));

    usage.oom_kills = None;
    assert_eq!(usage.validate(), Ok(()));

    usage.platform = Platform::LinuxX64;
    assert_eq!(usage.validate(), Err(UsageValidationError::InvalidOomKills));
}

#[test]
fn rejects_more_than_1000_steps_without_rejecting_the_points() {
    let mut usage = job_usage();
    let step = usage.steps[0].clone();
    usage.steps = vec![step.clone(); 1_000];
    assert_eq!(usage.validate(), Ok(()));

    usage.steps.push(step);
    assert_eq!(usage.validate_points(), Ok(()));
    assert_eq!(usage.validate(), Err(UsageValidationError::TooManySteps));
}

#[test]
fn rejects_step_names_over_256_characters_not_bytes() {
    let mut usage = job_usage();
    usage.steps[0].name = "é".repeat(256);
    assert_eq!(usage.validate(), Ok(()));

    usage.steps[0].name.push('é');
    assert_eq!(usage.validate(), Err(UsageValidationError::StepNameTooLong));
}
