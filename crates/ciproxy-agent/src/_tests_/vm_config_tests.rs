//! Boot drives contain the disposable job root without an unused Docker disk.

use crate::fixtures::vm;
use crate::vm_config::VmConfig;

#[test]
fn vm_boots_with_one_writable_root_drive() {
    let config = serde_json::to_value(VmConfig::for_vm(&vm("runner", false))).unwrap();
    assert_eq!(
        config["drives"],
        serde_json::json!([{
            "drive_id": "rootfs",
            "path_on_host": "/rootfs.ext4",
            "is_root_device": true,
            "is_read_only": false,
        }])
    );
}
