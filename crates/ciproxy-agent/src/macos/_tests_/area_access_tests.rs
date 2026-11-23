//! The area access check and the result file that install reads.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::area_access::{Access, clear_result, probe, read_result, record_result, wait_result};

/// A new folder under the system temporary folder, removed on drop.
struct Folder(PathBuf);

impl Folder {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("ciproxy-access-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).expect("folder");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Folder {
    fn drop(&mut self) {
        let _ = std::fs::set_permissions(&self.0, std::fs::Permissions::from_mode(0o700));
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn a_usable_area_is_open_and_keeps_no_check_file() {
    let area = Folder::new();
    assert_eq!(probe(area.path()).unwrap(), Access::Open);
    assert_eq!(std::fs::read_dir(area.path()).unwrap().count(), 0);
}

#[test]
fn a_refused_area_is_blocked() {
    if rustix::process::geteuid().is_root() {
        // Root passes the mode bits, so only macOS privacy control can
        // refuse it. The macOS run on the test Mac covers that case.
        return;
    }
    let area = Folder::new();
    std::fs::set_permissions(area.path(), std::fs::Permissions::from_mode(0o000)).unwrap();
    assert_eq!(probe(area.path()).unwrap(), Access::Blocked);
}

#[test]
fn a_missing_area_is_an_error_not_a_block() {
    let folder = Folder::new();
    assert!(probe(&folder.path().join("absent")).is_err());
}

#[test]
fn the_result_file_holds_the_last_check() {
    let control = Folder::new();
    let since = UNIX_EPOCH;
    assert_eq!(read_result(control.path(), since), None);
    record_result(control.path(), Access::Blocked).unwrap();
    assert_eq!(read_result(control.path(), since), Some(Access::Blocked));
    record_result(control.path(), Access::Open).unwrap();
    assert_eq!(
        wait_result(control.path(), since, Duration::ZERO, Duration::ZERO),
        Some(Access::Open)
    );
    clear_result(control.path()).unwrap();
    clear_result(control.path()).unwrap();
    assert_eq!(read_result(control.path(), since), None);
    for text in ["open", "unknown 1", "open soon"] {
        std::fs::write(control.path().join("area-access"), text).unwrap();
        assert_eq!(read_result(control.path(), since), None, "{text}");
    }
}

#[test]
fn a_result_of_an_older_process_is_not_used() {
    let control = Folder::new();
    record_result(control.path(), Access::Open).unwrap();
    let later = SystemTime::now() + Duration::from_secs(60);
    assert_eq!(read_result(control.path(), later), None);
    std::fs::write(control.path().join("area-access"), "blocked 100\n").unwrap();
    assert_eq!(
        read_result(control.path(), UNIX_EPOCH + Duration::from_millis(100)),
        Some(Access::Blocked)
    );
    assert_eq!(
        read_result(control.path(), UNIX_EPOCH + Duration::from_millis(101)),
        None
    );
}

#[test]
fn the_wait_ends_with_no_result_after_its_timeout() {
    let control = Folder::new();
    assert_eq!(
        wait_result(
            control.path(),
            UNIX_EPOCH,
            Duration::from_millis(30),
            Duration::from_millis(10)
        ),
        None
    );
}
