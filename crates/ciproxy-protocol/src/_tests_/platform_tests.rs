//! Wire values and VM limits of each platform.

use serde_json::json;

use crate::platform::{MACOS_VM_LIMIT, Platform, Virtualization};

#[test]
fn platforms_use_snake_case_wire_values() {
    for (platform, wire) in [
        (Platform::LinuxX64, "linux_x64"),
        (Platform::MacosArm64, "macos_arm64"),
    ] {
        assert_eq!(serde_json::to_value(platform).unwrap(), json!(wire));
        assert_eq!(
            serde_json::from_value::<Platform>(json!(wire)).unwrap(),
            platform
        );
    }
}

#[test]
fn only_macos_limits_the_number_of_vms() {
    assert_eq!(MACOS_VM_LIMIT, 2);
    assert_eq!(Platform::LinuxX64.vm_limit(), None);
    assert_eq!(Platform::MacosArm64.vm_limit(), Some(2));
}

#[test]
fn only_available_virtualization_allows_job_vms() {
    for (virtualization, wire, available) in [
        (Virtualization::Available, "available", true),
        (Virtualization::KvmUnavailable, "kvm_unavailable", false),
        (
            Virtualization::FrameworkUnsupported,
            "framework_unsupported",
            false,
        ),
        (
            Virtualization::EntitlementMissing,
            "entitlement_missing",
            false,
        ),
        (
            Virtualization::OperatingSystemTooOld,
            "operating_system_too_old",
            false,
        ),
    ] {
        assert_eq!(serde_json::to_value(virtualization).unwrap(), json!(wire));
        assert_eq!(virtualization.is_available(), available, "{wire}");
    }
}
