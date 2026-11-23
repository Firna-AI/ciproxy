//! Bundle paths and plain files.

use super::{Bundle, Identity};
use crate::error::Error;
use crate::mac_address::MacAddress;

#[test]
fn paths_sit_in_the_root() {
    let bundle = Bundle::new("/vms/a");
    assert_eq!(bundle.disk().to_str(), Some("/vms/a/disk.img"));
    assert_eq!(bundle.auxiliary_storage().to_str(), Some("/vms/a/aux.img"));
    assert_eq!(
        bundle.hardware_model().to_str(),
        Some("/vms/a/hardware-model.bin")
    );
    assert_eq!(
        bundle.machine_identifier().to_str(),
        Some("/vms/a/machine-identifier.bin")
    );
    assert_eq!(
        bundle.mac_address().to_str(),
        Some("/vms/a/mac-address.txt")
    );
}

#[test]
fn files_round_trip_and_a_complete_bundle_passes_the_check() {
    let directory = tempfile::tempdir().unwrap();
    let bundle = Bundle::new(directory.path().join("vm"));
    bundle.create_directory().unwrap();
    assert!(matches!(bundle.check_files(), Err(Error::Io { .. })));

    bundle.create_disk(64 * 1024 * 1024).unwrap();
    assert_eq!(bundle.disk().metadata().unwrap().len(), 64 * 1024 * 1024);
    assert!(bundle.create_disk(1).is_err());

    std::fs::write(bundle.auxiliary_storage(), b"aux").unwrap();
    bundle.write_hardware_model(b"model").unwrap();
    bundle.check_image_files().unwrap();
    assert!(matches!(
        bundle.check_files(),
        Err(Error::Io { path, .. }) if path == bundle.machine_identifier()
    ));
    bundle.write_machine_identifier(b"id").unwrap();
    assert!(matches!(
        bundle.check_files(),
        Err(Error::Io { path, .. }) if path == bundle.mac_address()
    ));
    let mac = MacAddress::new([0xa2, 0, 1, 2, 3, 4]);
    bundle.write_mac_address(mac).unwrap();

    assert_eq!(bundle.read_hardware_model().unwrap(), b"model");
    assert_eq!(bundle.read_machine_identifier().unwrap(), b"id");
    assert_eq!(bundle.read_mac_address().unwrap(), mac);
    assert_eq!(
        std::fs::read_to_string(bundle.mac_address()).unwrap(),
        "a2:00:01:02:03:04\n"
    );
    bundle.check_files().unwrap();

    let identity = Identity {
        mac_address: MacAddress::new([0xa2, 9, 9, 9, 9, 9]),
        machine_identifier: b"other".to_vec(),
    };
    bundle.write_identity(&identity).unwrap();
    assert_eq!(bundle.read_identity().unwrap(), identity);
}

#[test]
fn invalid_files_fail_with_the_path() {
    let directory = tempfile::tempdir().unwrap();
    let bundle = Bundle::new(directory.path());
    std::fs::write(bundle.mac_address(), "nope\n").unwrap();
    std::fs::write(bundle.hardware_model(), b"").unwrap();
    assert!(matches!(
        bundle.read_mac_address(),
        Err(Error::InvalidBundle { path, .. }) if path == bundle.mac_address()
    ));
    assert!(matches!(
        bundle.read_hardware_model(),
        Err(Error::InvalidBundle { .. })
    ));
    assert!(matches!(
        bundle.read_machine_identifier(),
        Err(Error::Io { .. })
    ));
}
