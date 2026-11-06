//! Dashboard root fit preserves prepared bytes and selected-image geometry.

use ciproxy_api::capacity::MachineCapacity;
use ciproxy_protocol::identifiers::ImageId;
use ciproxy_protocol::images::ImageMetadata;
use ciproxy_protocol::resources::Slots;
use ciproxy_store_interface::gateway::MachineSnapshot;
use ciproxy_store_interface::machines::Machine;
use uuid::Uuid;

use super::ReportedCapacity;

fn machine() -> MachineSnapshot {
    MachineSnapshot {
        machine: Machine {
            id: Uuid::from_u128(1),
            hostname: "capacity".to_owned(),
            processor: "x64".to_owned(),
            hardware_threads: 32,
            memory_bytes: 128_000_000_000,
            kvm_available: true,
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
            "hardware":{"processor":"x64","threads":32,"memory_bytes":128_000_000_000u64,"kvm":true,"disks":[]},
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
        },
        ImageMetadata {
            image_id: ImageId("local".to_owned()),
            root_disk_bytes: 59_000_000_000,
        },
    ];
    snapshot.report = Some(report);
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

    let report = snapshot.report.as_mut().unwrap();
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
