//! Contact freshness takes precedence over the selected image in the report.

use chrono::{DateTime, Duration, Utc};
use ciproxy_protocol::identifiers::ImageId;
use ciproxy_protocol::images::{ImageMetadata, ImageReport};
use ciproxy_protocol::platform::{Platform, Virtualization};
use ciproxy_protocol::reports::{DrainState, HardwareReport, MachineReport};
use ciproxy_protocol::resources::Slots;
use ciproxy_protocol::statistics::Statistics;
use uuid::Uuid;

use crate::gateway::{MachineSnapshot, MachineState, ReceivedReport};
use crate::machines::Machine;

#[test]
fn machine_state_requires_recent_contact_and_a_selected_reported_image() {
    let now: DateTime<Utc> = "2026-11-19T12:00:00Z".parse().unwrap();
    let image = ImageMetadata {
        image_id: ImageId("verified-image".to_owned()),
        root_disk_bytes: 60_000_000_000,
        free_job_bytes: None,
    };
    let selected = ImageReport {
        newest: Some(image.image_id.clone()),
        available: vec![image.clone()],
    };
    let empty = ImageReport {
        newest: None,
        available: vec![],
    };
    let unselected = ImageReport {
        newest: None,
        available: vec![image.clone()],
    };
    let unknown = ImageReport {
        newest: Some(ImageId("missing-image".to_owned())),
        available: vec![image],
    };
    for (name, age_millis, images, expected) in [
        (
            "no contact",
            None,
            Some(selected.clone()),
            MachineState::Offline,
        ),
        (
            "stale contact",
            Some(60_001),
            Some(selected.clone()),
            MachineState::Offline,
        ),
        (
            "exact boundary",
            Some(60_000),
            Some(selected.clone()),
            MachineState::Offline,
        ),
        ("no report", Some(0), None, MachineState::Preparing),
        ("no image", Some(0), Some(empty), MachineState::Preparing),
        (
            "no selection",
            Some(0),
            Some(unselected),
            MachineState::Preparing,
        ),
        (
            "missing selected image",
            Some(0),
            Some(unknown),
            MachineState::Preparing,
        ),
        (
            "recent contact",
            Some(59_999),
            Some(selected),
            MachineState::Ready,
        ),
    ] {
        let snapshot = MachineSnapshot {
            machine: Machine {
                id: Uuid::from_u128(1),
                hostname: "builder".to_owned(),
                processor: "x64".to_owned(),
                hardware_threads: 4,
                memory_bytes: 16_000_000_000,
                platform: Platform::LinuxX64,
                virtualization: Virtualization::KvmUnavailable,
                slots: 0,
                warm_runners: 0,
                draining: true,
                last_contact_at: age_millis.map(|age| now - Duration::milliseconds(age)),
                agent_version: "0.1.0".to_owned(),
                disks: vec![],
                removed_at: None,
            },
            report: images.map(|images| ReceivedReport {
                received_at: now - Duration::hours(1),
                report: MachineReport {
                    hostname: "builder".to_owned(),
                    agent_version: "0.1.0".to_owned(),
                    hardware: HardwareReport {
                        platform: Platform::LinuxX64,
                        processor: "x64".to_owned(),
                        threads: 4,
                        memory_bytes: 16_000_000_000,
                        virtualization: Virtualization::KvmUnavailable,
                        disks: vec![],
                    },
                    capacity: Slots(0),
                    job_disk_bytes: 0,
                    slots: vec![],
                    runners: vec![],
                    drain: DrainState::Draining,
                    cache: Statistics::Unavailable,
                    docker_disks: Statistics::Unavailable,
                    images,
                },
            }),
        };
        assert_eq!(snapshot.state(now), expected, "{name}");
    }
}
