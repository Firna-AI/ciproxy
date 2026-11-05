//! The complete job disk and measured overhead stay apart from the system remainder.

use crate::error::Error;
use crate::storage_sizing::{AreaSizing, DEFAULT_IMAGE_BYTES, setting};

#[test]
fn minimum_machine_keeps_its_full_job_disk_and_leaves_system_space() {
    let sizing = AreaSizing {
        host: 80_000_000_000,
        images: DEFAULT_IMAGE_BYTES,
    };
    assert_eq!(sizing.job(), 60_000_000_000);
    assert_eq!(sizing.length(100_000_000).unwrap(), 64_100_003_840);
    assert_eq!(
        sizing.measured(64_000_000_000, 63_900_000_000).unwrap(),
        65_991_778_304
    );
    assert_eq!(
        80_000_000_000 - sizing.length(100_000_000).unwrap(),
        15_899_996_160
    );
    assert!(matches!(
        AreaSizing {
            host: 80_000_000_000,
            images: 20_000_000_000
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
