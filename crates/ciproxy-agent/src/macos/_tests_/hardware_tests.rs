//! The Mac report names the first cause that stops job VMs.

use ciproxy_protocol::platform::{Platform, Virtualization};

use super::hardware::{
    AreaSpace, ImageNeeds, MINIMUM, MacFacts, MacosVersion, install_check, report, virtualization,
};
use crate::error::Error;

fn version(text: &str) -> MacosVersion {
    MacosVersion::parse(text).expect("version")
}

fn facts() -> MacFacts {
    MacFacts {
        processor: "Apple M4".to_owned(),
        threads: 10,
        memory_bytes: 16 << 30,
        version: version("26.7.3"),
        framework_supported: true,
        entitlement: true,
    }
}

#[test]
fn versions_parse_and_order_by_number() {
    assert_eq!(version("26.7").to_string(), "26.7.0");
    assert_eq!(version("26.7.3\n").to_string(), "26.7.3");
    assert!(version("26.10") > version("26.9.9"));
    assert!(version("26.7.2") == MINIMUM);
    for text in ["", "26.", "26.x", "26.7.3.1", "v26"] {
        assert_eq!(MacosVersion::parse(text), None, "{text}");
    }
}

#[test]
fn virtualization_names_the_first_blocking_cause() {
    let image = |minimum: &str, supported: bool| ImageNeeds {
        minimum_host: version(minimum),
        hardware_model_supported: supported,
    };
    assert_eq!(virtualization(&facts(), None), Virtualization::Available);
    assert_eq!(
        virtualization(&facts(), Some(image("26.7.3", true))),
        Virtualization::Available
    );
    let cases = [
        (
            MacFacts {
                framework_supported: false,
                entitlement: false,
                ..facts()
            },
            None,
            Virtualization::FrameworkUnsupported,
        ),
        (
            MacFacts {
                entitlement: false,
                ..facts()
            },
            None,
            Virtualization::EntitlementMissing,
        ),
        (
            MacFacts {
                version: version("26.6.2"),
                ..facts()
            },
            None,
            Virtualization::OperatingSystemTooOld,
        ),
        (
            facts(),
            Some(image("27.0", true)),
            Virtualization::OperatingSystemTooOld,
        ),
        (
            facts(),
            Some(image("26.0", false)),
            Virtualization::FrameworkUnsupported,
        ),
    ];
    for (facts, image, expected) in cases {
        assert_eq!(
            virtualization(&facts, image),
            expected,
            "{facts:?} {image:?}"
        );
    }
}

#[test]
fn the_report_names_the_area_volume_and_caps_free_space() {
    let area = AreaSpace {
        capacity_bytes: 400 << 30,
        free_bytes: 500 << 30,
    };
    let hardware = report(&facts(), Virtualization::Available, Some(area));
    assert_eq!(hardware.platform, Platform::MacosArm64);
    assert_eq!(hardware.processor, "Apple M4");
    assert_eq!((hardware.threads, hardware.memory_bytes), (10, 16 << 30));
    assert_eq!(hardware.disks.len(), 1);
    assert_eq!(hardware.disks[0].name, "/var/db/ciproxy/area");
    assert_eq!(hardware.disks[0].free_bytes, 400 << 30);
    assert!(
        report(&facts(), Virtualization::Available, None)
            .disks
            .is_empty()
    );
}

#[test]
fn install_refuses_with_a_repair_message() {
    assert!(install_check(true, true, version("26.7.2")).is_ok());
    assert!(matches!(
        install_check(false, true, version("26.7.3")),
        Err(Error::MacHostUnsupported)
    ));
    assert!(matches!(
        install_check(true, false, version("26.7.3")),
        Err(Error::MacHostUnsupported)
    ));
    let error = install_check(true, true, version("15.4")).expect_err("too old");
    assert_eq!(
        error.to_string(),
        "[agent/host] ciproxy needs macOS 26.7.2 or later on this Mac. This Mac runs macOS 15.4.0. Update macOS in System Settings, then run the install again."
    );
}
