//! The complete job disk and measured overhead stay apart from the system remainder.

use crate::error::Error;
use crate::storage_sizing::{AreaSizing, DEFAULT_IMAGE_BYTES, setting};

#[test]
fn minimum_machine_keeps_its_full_job_disk_and_leaves_system_space() {
    let sizing = AreaSizing {
        host: 250_000_000_000,
        images: DEFAULT_IMAGE_BYTES,
    };
    assert_eq!(sizing.job(), 82_500_000_000);
    assert_eq!(sizing.length(100_000_000).unwrap(), 222_600_003_584);
    assert_eq!(
        sizing.measured(222_500_000_000, 222_400_000_000).unwrap(),
        225_194_905_600
    );
    assert_eq!(
        250_000_000_000 - sizing.length(100_000_000).unwrap(),
        27_399_996_416
    );
    assert!(matches!(
        AreaSizing {
            host: 250_000_000_000,
            images: 250_000_000_000
        }
        .length(0),
        Err(Error::Preparation {
            component: "image budget",
            ..
        })
    ));
    assert!(matches!(
        AreaSizing {
            host: u64::MAX,
            images: u64::MAX
        }
        .length(100),
        Err(Error::Preparation { .. })
    ));
}

#[test]
fn image_budget_override_changes_only_the_disk_left_for_host_and_jobs() {
    let sizing = AreaSizing {
        host: 250_000_000_000,
        images: 160_000_000_000,
    };
    assert_eq!(sizing.job(), 67_500_000_000);
    assert_eq!(sizing.length(0).unwrap(), 227_500_003_328);
    assert_eq!(
        AreaSizing {
            host: 100_000_000,
            images: 140_000_000_000,
        }
        .job(),
        0
    );
}

#[test]
fn explicit_budget_values_are_checked_without_ignoring_invalid_configuration() {
    assert_eq!(setting(None).unwrap(), None);
    assert_eq!(setting(Some("4000000000")).unwrap(), Some(4_000_000_000));
    for value in ["0", "16777216", "-1", "4GB", "18446744073709551616"] {
        assert!(
            matches!(
                setting(Some(value)),
                Err(Error::Preparation {
                    component: "image budget setting",
                    ..
                })
            ),
            "{value}"
        );
    }
}
