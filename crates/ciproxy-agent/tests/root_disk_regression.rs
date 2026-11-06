//! Opt-in real ext4 growth through the production reflink preparation boundary.

#![expect(
    dead_code,
    reason = "Production adapters expose more operations than this regression needs."
)]

#[path = "../src/error.rs"]
mod error;
#[path = "../src/process.rs"]
mod process;
#[path = "../src/vm_root.rs"]
mod vm_root;

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use ciproxy_agent::root_sizing::RootSizing;

use process::LinuxCommands;
use vm_root::{LinuxRootFiles, ReflinkRoot, RootDisk};

fn run(program: &str, args: &[&str]) -> String {
    let output = Command::new(program)
        .args(args)
        .env("LC_ALL", "C")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{program}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

struct Mount(PathBuf);

struct PublicMountpoint(PathBuf);

impl PublicMountpoint {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("ciproxy-root-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for PublicMountpoint {
    fn drop(&mut self) {
        fs::remove_dir(&self.0).unwrap();
    }
}

impl Mount {
    fn new(image: &Path, path: &Path) -> Self {
        run(
            "sudo",
            &[
                "-n",
                "mount",
                "-t",
                "ext4",
                "-o",
                "loop",
                "--",
                image.to_str().unwrap(),
                path.to_str().unwrap(),
            ],
        );
        Self(path.to_owned())
    }

    fn available(&self) -> u64 {
        run(
            "sudo",
            &[
                "-n",
                "setpriv",
                "--reuid=1001",
                "--regid=1001",
                "--clear-groups",
                "df",
                "--block-size=1",
                "--output=avail",
                self.0.to_str().unwrap(),
            ],
        )
        .split_whitespace()
        .last()
        .unwrap()
        .parse()
        .unwrap()
    }
}

impl Drop for Mount {
    fn drop(&mut self) {
        run("sudo", &["-n", "umount", "--", self.0.to_str().unwrap()]);
    }
}

#[tokio::test]
async fn grown_roots_keep_label_free_bytes_available_to_the_runner_with_one_percent_reserve() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.context/ciproxy-mvp-77d-free-disk")
        .join(format!("ext4-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let preparation = ReflinkRoot {
        files: Arc::new(LinuxRootFiles),
        commands: Arc::new(LinuxCommands),
    };
    for (image_bytes, free_bytes) in [
        (64 * 1024 * 1024, 1_000_000_000),
        (256 * 1024 * 1024, 20_000_000_000),
    ] {
        let base = root.join(format!("base-{image_bytes}.ext4"));
        let copy = root.join(format!("job-{image_bytes}.ext4"));
        let directory = PublicMountpoint::new();
        let mountpoint = &directory.0;
        OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&base)
            .unwrap()
            .set_len(image_bytes)
            .unwrap();
        run(
            "mkfs.ext4",
            &[
                "-q",
                "-F",
                "-m",
                "1",
                "-b",
                "4096",
                "-I",
                "256",
                "-i",
                "16384",
                base.to_str().unwrap(),
            ],
        );
        {
            let mounted = Mount::new(&base, mountpoint);
            run(
                "sudo",
                &["-n", "chmod", "777", mountpoint.to_str().unwrap()],
            );
            let mut payload = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(mountpoint.join("image-software"))
                .unwrap();
            let bytes = mounted.available().saturating_sub(1024 * 1024) / 4096 * 4096;
            let block = [b'I'; 4096];
            for _ in 0..bytes / 4096 {
                payload.write_all(&block).unwrap();
            }
            payload.sync_all().unwrap();
            assert!(mounted.available() < 2 * 1024 * 1024);
        }
        run("e2fsck", &["-f", "-p", base.to_str().unwrap()]);
        let before = run("sha256sum", &["--", base.to_str().unwrap()]);
        let root_bytes = RootSizing {
            image_root_bytes: image_bytes,
        }
        .root_bytes(free_bytes)
        .unwrap();
        preparation.prepare(&base, &copy, root_bytes).await.unwrap();
        assert_eq!(copy.metadata().unwrap().len(), root_bytes);
        let available = {
            let mounted = Mount::new(&copy, mountpoint);
            mounted.available()
        };
        assert!(
            available >= free_bytes,
            "image={image_bytes}, root={root_bytes}, free={free_bytes}, available={available}"
        );
        assert_eq!(run("sha256sum", &["--", base.to_str().unwrap()]), before);
        println!(
            "image_bytes={image_bytes} free_job_bytes={free_bytes} root_bytes={root_bytes} runner_uid=1001 available_bytes={available} reserve_percent=1 base_unchanged=true"
        );
        fs::remove_file(&base).unwrap();
        fs::remove_file(&copy).unwrap();
    }
    fs::remove_dir(&root).unwrap();
}
