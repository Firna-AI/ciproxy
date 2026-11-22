//! A helper's stop maps to the unit states that the guest bridge knows, and
//! a new helper is awaited until its control socket answers.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use unimock::{MockFn, Unimock, matching};

use super::guests::{GUEST_PORT, VmFolders, unit_state, wait_for_helper};
use super::vm_control::VmPhase;
use super::vm_control_server::{VmControlMock, bind, serve};
use super::vm_spec::ExitReason;
use crate::error::Error;
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

/// A short area folder under the system temporary folder, removed on drop.
/// Socket paths have a length limit, so the names stay short.
struct Area(PathBuf);

impl Drop for Area {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const HELPER: &str = "vm1";

fn helper_area() -> (Area, VmFolders) {
    let suffix = uuid::Uuid::new_v4().simple().to_string();
    let area = Area(std::env::temp_dir().join(format!("cg-{}", &suffix[..12])));
    let folders = VmFolders {
        area: area.0.clone(),
    };
    std::fs::create_dir_all(folders.folder(HELPER)).expect("VM folder");
    (area, folders)
}

/// The helper binds its socket only some time after `launchctl bootstrap`.
#[tokio::test]
async fn a_new_helper_is_awaited_until_its_socket_answers() {
    let (_area, folders) = helper_area();
    let id = HELPER;
    let socket = folders.client(id).socket;
    let helper = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(300)).await;
        let listener = bind(&socket).expect("bind");
        let control = Unimock::new(
            VmControlMock::phase
                .each_call(matching!())
                .returns(VmPhase::Running),
        );
        serve(listener, Arc::new(control)).await
    });

    let started = Instant::now();
    wait_for_helper(&folders, id, Duration::from_secs(10))
        .await
        .expect("the helper answers");

    assert!(started.elapsed() >= Duration::from_millis(300));
    helper.abort();
}

#[tokio::test]
async fn a_helper_that_already_ended_fails_the_wait_at_once() {
    let (_area, folders) = helper_area();
    let id = HELPER;
    std::fs::write(folders.exit_file(id), br#"{"reason":"failure"}"#).expect("exit file");

    let started = Instant::now();
    let result = wait_for_helper(&folders, id, Duration::from_secs(10)).await;

    assert!(matches!(result, Err(Error::Disconnected)), "{result:?}");
    assert!(started.elapsed() < Duration::from_secs(1));
}

#[tokio::test]
async fn a_helper_that_never_answers_fails_after_the_wait() {
    let (_area, folders) = helper_area();
    let id = HELPER;

    let result = wait_for_helper(&folders, id, Duration::from_millis(300)).await;

    assert!(matches!(result, Err(Error::VmHelper { .. })), "{result:?}");
}
