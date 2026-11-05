//! The production mount inspector refuses foreign backing files and unsafe options.

use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::area::AreaMounts;
use crate::area_mounts::LinuxAreaMounts;
use crate::error::Error;
use crate::host_paths::{AREA, BACKING, MOUNT};
use crate::host_system::{HostOutput, HostSystemMock};

#[test]
fn only_the_owned_read_write_xfs_loop_mount_is_accepted() {
    for (options, kind, backing, accepted) in [
        ("rw,nosuid", "xfs", BACKING, true),
        ("rw,nodev", "xfs", BACKING, false),
        ("rw,noexec", "xfs", BACKING, false),
        ("ro,nosuid", "xfs", BACKING, false),
        ("rw,nosuid", "ext4", BACKING, false),
        ("rw,nosuid", "xfs", "/foreign/data.img", false),
    ] {
        let line = format!("83 25 7:3 / {AREA} {options} - {kind} /dev/loop3 rw\n");
        let mock = Arc::new(Unimock::new((
            HostSystemMock::read
                .some_call(matching!("/proc/self/mountinfo"))
                .returns(Ok(line.into_bytes())),
            HostSystemMock::read
                .some_call(matching!("/sys/class/block/loop3/loop/backing_file"))
                .answers_arc(Arc::new(move |_, _| Ok(backing.as_bytes().to_vec())))
                .at_least_times(0),
        )));
        let result = LinuxAreaMounts {
            system: mock.clone(),
        }
        .mounted();
        let actual = match result {
            Ok(true) => "owned",
            Err(Error::Preparation {
                component: "XFS loop mount",
                ..
            }) => "refused",
            _ => "unexpected",
        };
        assert_eq!(
            actual,
            if accepted { "owned" } else { "refused" },
            "{options} {kind} {backing}"
        );
    }
}

#[tokio::test]
async fn remaining_loop_users_block_cleanup_after_the_mount_disappears() {
    for (attached, present, expected, programs, first_args) in [
        (
            false,
            true,
            "clear",
            vec!["losetup", "systemctl"],
            vec!["--associated", BACKING],
        ),
        (
            true,
            true,
            "busy",
            vec!["losetup"],
            vec!["--associated", BACKING],
        ),
        (
            false,
            false,
            "clear",
            vec!["systemctl"],
            vec!["disable", MOUNT],
        ),
    ] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let recorded = calls.clone();
        let mock = Arc::new(Unimock::new((
            HostSystemMock::read
                .some_call(matching!("/proc/self/mountinfo"))
                .returns(Ok(Vec::new())),
            HostSystemMock::execute
                .each_call(matching!(_, _))
                .answers_arc(Arc::new(move |_, program, args| {
                    recorded
                        .lock()
                        .unwrap()
                        .push((program.to_owned(), args.to_vec()));
                    let output = if attached && program == "losetup" {
                        b"/dev/loop9: remaining user\n".to_vec()
                    } else {
                        Vec::new()
                    };
                    Ok(HostOutput {
                        code: 0,
                        stdout: output,
                    })
                })),
        )));
        let result = LinuxAreaMounts {
            system: mock.clone(),
        }
        .stop(present)
        .await;
        let outcome = match result {
            Ok(()) => "clear",
            Err(Error::Preparation {
                component: "loop device users",
                ..
            }) => "busy",
            _ => "unexpected",
        };
        assert_eq!(outcome, expected);
        let calls = calls.lock().unwrap();
        assert_eq!(calls[0].1, first_args);
        assert_eq!(
            calls
                .iter()
                .map(|(program, _)| program.as_str())
                .collect::<Vec<_>>(),
            programs
        );
    }
}
