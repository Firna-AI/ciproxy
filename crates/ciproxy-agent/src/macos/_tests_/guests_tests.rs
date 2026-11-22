//! A helper's stop maps to the unit states that the guest bridge knows.

use std::path::PathBuf;

use super::guests::{GUEST_PORT, VmFolders, unit_state};
use super::vm_spec::ExitReason;
use crate::unit_state::UnitState;

#[test]
fn stop_reasons_map_to_unit_states() {
    assert_eq!(unit_state(ExitReason::GuestShutdown), UnitState::Exited(0));
    assert_eq!(unit_state(ExitReason::Failure), UnitState::Exited(1));
    assert_eq!(unit_state(ExitReason::HostStop), UnitState::Absent);
    assert_eq!(GUEST_PORT, 5252);
}

#[test]
fn helper_files_stay_in_the_vm_folder() {
    let folders = VmFolders {
        area: PathBuf::from("/var/db/ciproxy/area"),
    };
    let id = "0123456789abcdef0123456789abcdef";
    let socket = folders.client(id).socket;
    assert_eq!(
        socket,
        PathBuf::from(format!("/var/db/ciproxy/area/vms/{id}/control.sock"))
    );
    assert!(socket.as_os_str().len() < 104, "macOS limits socket paths");
    assert_eq!(
        folders.exit_file(id),
        PathBuf::from(format!("/var/db/ciproxy/area/vms/{id}/exit.json"))
    );
}
