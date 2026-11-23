//! Dashboard root fit preserves prepared bytes and selected-image geometry.

use ciproxy_api::capacity::MachineCapacity;
use ciproxy_protocol::identifiers::ImageId;
use ciproxy_protocol::images::ImageMetadata;
use ciproxy_protocol::platform::{Platform, Virtualization};
use ciproxy_protocol::resources::Slots;
use ciproxy_protocol::statistics::Statistics;
use ciproxy_store_interface::gateway::{MachineSnapshot, ReceivedReport};
use ciproxy_store_interface::machines::Machine;
use uuid::Uuid;

use ciproxy_agent::root_sizing::MACOS_ROOT_MARGIN_BYTES;

use super::ReportedCapacity;

fn machine() -> MachineSnapshot {
    MachineSnapshot {
        machine: Machine {
            id: Uuid::from_u128(1),
            hostname: "capacity".to_owned(),
            processor: "x64".to_owned(),
            hardware_threads: 32,
            memory_bytes: 128_000_000_000,
            platform: Platform::LinuxX64,
            virtualization: Virtualization::Available,
            slots: 12,
            warm_runners: 0,
            draining: false,
            last_contact_at: None,
            agent_version: "fixture".to_owned(),
            disks: vec![],
            removed_at: None,
        },
        report: None,
    }
}

#[test]
fn dashboard_capacity_subtracts_only_active_leases_and_sizes_the_selected_local_image() {
    let mut snapshot = machine();
    let mut report = serde_json::from_value::<ciproxy_protocol::reports::MachineReport>(
        serde_json::json!({
            "hostname":"capacity", "agent_version":"fixture",
            "hardware":{"platform":"linux_x64","processor":"x64","threads":32,"memory_bytes":128_000_000_000u64,"virtualization":"available","disks":[]},
            "capacity":7,"job_disk_bytes":150_000_000_000u64,"slots":[],"runners":[],"drain":"active",
            "cache":{"used_bytes":8_000_000_000u64,"capacity_bytes":9_000_000_000u64,"restores_last_7_days":0,"hits_last_7_days":0,"jobs":[]},
            "docker_disks":{"leased_bytes":13_000_000_000u64,"repositories":1,"used_bytes":80_000_000_000u64},
            "images":{"available":[],"newest":"local"}
        }),
    ).unwrap();
    report.images.available = vec![
        ImageMetadata {
            image_id: ImageId("downloading".to_owned()),
            root_disk_bytes: 71_000_000_000,
            free_job_bytes: None,
        },
        ImageMetadata {
            image_id: ImageId("local".to_owned()),
            root_disk_bytes: 59_000_000_000,
            free_job_bytes: None,
        },
    ];
    snapshot.report = Some(ReceivedReport {
        report,
        received_at: chrono::DateTime::UNIX_EPOCH,
    });
    let adapter: &dyn MachineCapacity = &ReportedCapacity;
    let capacity = adapter.capacity(&snapshot);
    assert_eq!(capacity.slots, Slots(7));
    assert_eq!(capacity.job_disk_bytes, Some(137_000_000_000));
    assert_eq!(capacity.image_disk_bytes, Some(59_000_000_000));
    assert_eq!(
        adapter.required_root_bytes(&snapshot, 20_000_000_000),
        Some(79_635_853_312)
    );
    assert_eq!(adapter.required_root_bytes(&snapshot, u64::MAX), None);

    snapshot.report.as_mut().unwrap().report.cache = Statistics::Unavailable;
    assert_eq!(
        adapter.capacity(&snapshot).job_disk_bytes,
        Some(137_000_000_000)
    );
    let measured_docker = snapshot
        .report
        .as_ref()
        .unwrap()
        .report
        .docker_disks
        .clone();
    snapshot.report.as_mut().unwrap().report.docker_disks = Statistics::Unavailable;
    let capacity = adapter.capacity(&snapshot);
    assert_eq!(capacity.job_disk_bytes, None);
    assert_eq!(capacity.slots, Slots(7));
    assert_eq!(capacity.image_disk_bytes, Some(59_000_000_000));

    let report = &mut snapshot.report.as_mut().unwrap().report;
    report.docker_disks = measured_docker;
    report.job_disk_bytes = 0;
    report.images.newest = Some(ImageId("unavailable".to_owned()));
    let capacity = adapter.capacity(&snapshot);
    assert_eq!(capacity.job_disk_bytes, Some(0));
    assert_eq!(capacity.image_disk_bytes, None);
    assert_eq!(adapter.required_root_bytes(&snapshot, 20_000_000_000), None);
}

#[test]
fn dashboard_capacity_without_a_report_remains_unavailable() {
    let snapshot = machine();
    let adapter: &dyn MachineCapacity = &ReportedCapacity;
    let capacity = adapter.capacity(&snapshot);
    assert_eq!(capacity.slots, Slots(0));
    assert_eq!(capacity.job_disk_bytes, None);
    assert_eq!(capacity.image_disk_bytes, None);
    assert_eq!(adapter.required_root_bytes(&snapshot, 20_000_000_000), None);
}

/// A Mac sends no Docker report, and its VM is an APFS clone of the image.
#[test]
fn a_mac_counts_its_job_disk_and_the_label_disk_of_each_vm() {
    let mut snapshot = machine();
    snapshot.machine.platform = Platform::MacosArm64;
    let mut report = serde_json::from_value::<ciproxy_protocol::reports::MachineReport>(
        serde_json::json!({
            "hostname":"capacity", "agent_version":"fixture",
            "hardware":{"platform":"macos_arm64","processor":"Apple M4","threads":10,"memory_bytes":16_000_000_000u64,"virtualization":"available","disks":[]},
            "capacity":1,"job_disk_bytes":120_000_000_000u64,"slots":[],"runners":[],"drain":"active",
            "cache":{"status":"unavailable"},"docker_disks":{"status":"unavailable"},
            "images":{"available":[],"newest":"macos"}
        }),
    )
    .unwrap();
    report.images.available = vec![ImageMetadata {
        image_id: ImageId("macos".to_owned()),
        root_disk_bytes: 200_000_000_000,
        free_job_bytes: Some(121_000_000_000),
    }];
    snapshot.report = Some(ReceivedReport {
        report,
        received_at: chrono::DateTime::UNIX_EPOCH,
    });
    let adapter: &dyn MachineCapacity = &ReportedCapacity;

    let capacity = adapter.capacity(&snapshot);
    assert_eq!(capacity.job_disk_bytes, Some(120_000_000_000));
    assert_eq!(
        adapter.required_root_bytes(&snapshot, 20_000_000_000),
        Some(20_000_000_000 + MACOS_ROOT_MARGIN_BYTES)
    );
    assert_eq!(
        adapter.required_root_bytes(&snapshot, 121_000_000_001),
        None
    );
}
