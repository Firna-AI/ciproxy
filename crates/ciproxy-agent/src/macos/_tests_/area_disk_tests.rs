//! The area disk setting, the disk checks, the record, and the uninstall
//! decision.

use std::path::PathBuf;

use super::area_disk::{
    AreaDisk, AreaRecord, Deletion, DiskFacts, area_mounted, check_disk, deletion,
};
use crate::error::Error;

#[test]
fn the_setting_names_a_path_or_a_container() {
    assert_eq!(AreaDisk::parse(None).unwrap(), AreaDisk::Startup);
    assert_eq!(AreaDisk::parse(Some("  ")).unwrap(), AreaDisk::Startup);
    assert_eq!(
        AreaDisk::parse(Some("/Volumes/cals-sandisk")).unwrap(),
        AreaDisk::Path(PathBuf::from("/Volumes/cals-sandisk"))
    );
    assert_eq!(
        AreaDisk::parse(Some(" disk5\n")).unwrap(),
        AreaDisk::Container("disk5".to_owned())
    );
    assert_eq!(
        AreaDisk::parse(Some("disk12")).unwrap().setting(),
        Some("disk12".to_owned())
    );
    assert_eq!(AreaDisk::Startup.setting(), None);
}

#[test]
fn the_setting_refuses_other_values() {
    for value in ["disk", "disk5s1", "Volumes/x", "smb://host/share", "/a\nb"] {
        assert!(
            matches!(
                AreaDisk::parse(Some(value)),
                Err(Error::Preparation {
                    component: "area disk setting",
                    ..
                })
            ),
            "{value:?}"
        );
    }
}

fn internal_apfs() -> DiskFacts {
    DiskFacts {
        filesystem: Some("apfs".to_owned()),
        writable_volume: Some(true),
        writable_media: true,
        internal: true,
        removable: false,
        protocol: "Apple Fabric".to_owned(),
    }
}

fn refused(facts: &DiskFacts) -> bool {
    matches!(
        check_disk(facts),
        Err(Error::Preparation {
            component: "area disk",
            ..
        })
    )
}

#[test]
fn an_internal_writable_apfs_disk_passes_with_no_warning() {
    assert!(!check_disk(&internal_apfs()).unwrap());
    let container = DiskFacts {
        filesystem: None,
        writable_volume: None,
        ..internal_apfs()
    };
    assert!(!check_disk(&container).unwrap());
}

#[test]
fn an_external_or_removable_disk_passes_with_a_warning() {
    let usb = DiskFacts {
        internal: false,
        protocol: "USB".to_owned(),
        ..internal_apfs()
    };
    assert!(check_disk(&usb).unwrap());
    let removable = DiskFacts {
        removable: true,
        ..internal_apfs()
    };
    assert!(check_disk(&removable).unwrap());
}

#[test]
fn other_filesystems_read_only_disks_and_disk_images_are_refused() {
    let exfat = DiskFacts {
        filesystem: Some("exfat".to_owned()),
        ..internal_apfs()
    };
    let read_only_volume = DiskFacts {
        writable_volume: Some(false),
        ..internal_apfs()
    };
    let read_only_media = DiskFacts {
        writable_media: false,
        ..internal_apfs()
    };
    let image = DiskFacts {
        protocol: "Disk Image".to_owned(),
        ..internal_apfs()
    };
    for facts in [exfat, read_only_volume, read_only_media, image] {
        assert!(refused(&facts), "{facts:?}");
    }
}

fn record(container_uuid: Option<&str>) -> AreaRecord {
    AreaRecord {
        volume_uuid: "7144464F-C070-43D5-BEEF-0DF23584DDD5".to_owned(),
        quota_bytes: 42_949_672_960,
        container_uuid: container_uuid.map(str::to_owned),
        disk: None,
        external: false,
    }
}

#[test]
fn an_older_record_without_a_container_still_loads() {
    let old = r#"{"volume_uuid":"7144464F-C070-43D5-BEEF-0DF23584DDD5","quota_bytes":42949672960}"#;
    let loaded: AreaRecord = serde_json::from_str(old).unwrap();
    assert_eq!(loaded, record(None));
    assert_eq!(serde_json::to_string(&loaded).unwrap(), old);
    let new = AreaRecord {
        disk: Some("/Volumes/cals-sandisk".to_owned()),
        external: true,
        ..record(Some("C-UUID"))
    };
    let text = serde_json::to_string(&new).unwrap();
    assert_eq!(serde_json::from_str::<AreaRecord>(&text).unwrap(), new);
}

#[test]
fn uninstall_deletes_only_the_named_volume_in_the_recorded_container() {
    let recorded = record(Some("c-uuid"));
    assert_eq!(
        deletion(&recorded, Some(("ciproxy", Some("C-UUID"))), "ciproxy"),
        Deletion::Delete
    );
    assert_eq!(deletion(&recorded, None, "ciproxy"), Deletion::Missing);
    assert_eq!(
        deletion(&recorded, Some(("ciproxy", None)), "ciproxy"),
        Deletion::Unknown,
        "an unreadable container keeps the volume and the record"
    );
    for found in [
        ("ciproxy", Some("other")),
        ("Data", Some("c-uuid")),
        ("Data", None),
    ] {
        assert_eq!(
            deletion(&recorded, Some(found), "ciproxy"),
            Deletion::Foreign,
            "{found:?}"
        );
    }
    assert_eq!(
        deletion(&record(None), Some(("ciproxy", None)), "ciproxy"),
        Deletion::Delete
    );
}

#[test]
fn a_bare_folder_is_not_a_mounted_area() {
    let folder = std::env::temp_dir().join(format!("ciproxy-area-{}", uuid::Uuid::new_v4()));
    let area = folder.join("area");
    assert!(!area_mounted(&area), "a missing folder");
    std::fs::create_dir_all(&area).unwrap();
    assert!(!area_mounted(&area), "a folder on the parent device");
    std::fs::remove_dir_all(&folder).unwrap();
}
