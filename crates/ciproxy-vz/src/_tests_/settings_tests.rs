//! Settings checks against limits.

use super::{Display, Limits, VmSettings};

const GIB: u64 = 1024 * 1024 * 1024;

fn limits() -> Limits {
    Limits {
        min_cpus: 2,
        max_cpus: 8,
        min_memory_bytes: 2 * GIB,
        max_memory_bytes: 16 * GIB,
    }
}

fn settings() -> VmSettings {
    VmSettings {
        cpus: 4,
        memory_bytes: 4 * GIB,
        display: None,
    }
}

#[test]
fn settings_inside_the_limits_pass() {
    settings().check(limits()).unwrap();
    let mut with_display = settings();
    with_display.display = Some(Display::default());
    with_display.check(limits()).unwrap();
}

#[test]
fn settings_outside_the_limits_fail() {
    let cases = [
        VmSettings {
            cpus: 1,
            ..settings()
        },
        VmSettings {
            cpus: 9,
            ..settings()
        },
        VmSettings {
            memory_bytes: GIB,
            ..settings()
        },
        VmSettings {
            memory_bytes: 17 * GIB,
            ..settings()
        },
        VmSettings {
            memory_bytes: 4 * GIB + 1,
            ..settings()
        },
        VmSettings {
            display: Some(Display {
                width: 0,
                ..Display::default()
            }),
            ..settings()
        },
    ];
    for case in cases {
        assert!(case.check(limits()).is_err(), "{case:?}");
    }
}

#[test]
fn and_takes_the_narrower_limits() {
    let guest = Limits {
        min_cpus: 4,
        max_cpus: 32,
        min_memory_bytes: GIB,
        max_memory_bytes: 8 * GIB,
    };
    assert_eq!(
        limits().and(guest),
        Limits {
            min_cpus: 4,
            max_cpus: 8,
            min_memory_bytes: 2 * GIB,
            max_memory_bytes: 8 * GIB,
        }
    );
}
