//! Explicit root-only filesystem regression. The wrapper supplies isolated mounts.

#![expect(
    dead_code,
    unused_imports,
    reason = "The opt-in harness imports production modules and their fake-only tests. It exercises only real filesystem entry points."
)]

#[path = "../src/budget_file.rs"]
mod budget_file;
#[path = "../src/error.rs"]
mod error;
#[path = "../src/host_system.rs"]
mod host_system;
#[path = "../src/image_space.rs"]
mod image_space;
#[path = "../src/sparse_copy.rs"]
mod sparse_copy;
#[path = "../src/storage_sizing.rs"]
mod storage_sizing;

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use sha2::{Digest, Sha256};

use budget_file::BudgetFile;
use error::Error;
use image_space::{ImageBudget, ImageSpace, LinuxSpaceUsage};
use storage_sizing::AreaSizing;

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

fn digest(path: &Path) -> String {
    let bytes = fs::read(path).unwrap();
    hex::encode(Sha256::digest(bytes))
}

fn main() {
    let root = PathBuf::from(
        std::env::var_os("CIPROXY_REGRESSION_ROOT").expect("use the filesystem regression wrapper"),
    );
    let host = root.join("host");
    let area = host.join("area");
    let backing = host.join("area.xfs");
    let host_bytes = fs2::statvfs(&host).unwrap().total_space();
    let sizing = AreaSizing {
        host: host_bytes,
        images: 256 * 1024 * 1024,
    };
    let file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&backing)
        .unwrap();
    file.set_len(sizing.length(0).unwrap()).unwrap();
    run(
        "mkfs.xfs",
        &[
            "-q",
            "-b",
            "size=4096",
            "-m",
            "reflink=1",
            "--",
            backing.to_str().unwrap(),
        ],
    );
    fs::create_dir(&area).unwrap();
    run(
        "mount",
        &[
            "-t",
            "xfs",
            "-o",
            "loop,nosuid",
            "--",
            backing.to_str().unwrap(),
            area.to_str().unwrap(),
        ],
    );
    fs::create_dir(area.join("images")).unwrap();
    fs::create_dir(area.join("jailer")).unwrap();
    for _ in 0..4 {
        let available = fs2::statvfs(&area).unwrap().available_space();
        let target = sizing
            .measured(file.metadata().unwrap().len(), available)
            .unwrap();
        if target <= file.metadata().unwrap().len() {
            break;
        }
        file.set_len(target).unwrap();
        file.sync_all().unwrap();
        let device = run(
            "findmnt",
            &["-n", "-o", "SOURCE", "--mountpoint", area.to_str().unwrap()],
        );
        run("losetup", &["--set-capacity", device.trim()]);
        run("xfs_growfs", &[area.to_str().unwrap()]);
    }
    assert!(
        fs2::statvfs(&area).unwrap().available_space()
            >= sizing.job() + sizing.images + sizing.metadata_safety()
    );
    assert!(file.metadata().unwrap().len() < host_bytes);
    let space: Arc<dyn ImageSpace> = Arc::new(ImageBudget::new(
        sizing.images,
        Arc::new(LinuxSpaceUsage { area: area.clone() }),
    ));
    let base = area.join("images/base");
    let mut output = BudgetFile::create(&base, space.clone()).unwrap();
    let block = vec![b'B'; 1024 * 1024];
    for _ in 0..128 {
        output.write(&block).unwrap();
    }
    output.sync().unwrap();
    let before = digest(&base);
    let vm_root = area.join("jailer/firecracker/0123456789abcdef0123456789abcdef/root");
    fs::create_dir_all(&vm_root).unwrap();
    let vm = vm_root.join("rootfs.ext4");
    run(
        "cp",
        &[
            "--reflink=always",
            "--",
            base.to_str().unwrap(),
            vm.to_str().unwrap(),
        ],
    );
    let mut disk = OpenOptions::new().write(true).open(&vm).unwrap();
    disk.set_len(sizing.job()).unwrap();
    assert_ne!(base.metadata().unwrap().ino(), vm.metadata().unwrap().ino());
    let shared = run(
        "filefrag",
        &["-v", "--", base.to_str().unwrap(), vm.to_str().unwrap()],
    );
    assert!(shared.contains("shared"));
    let block = vec![b'J'; 1024 * 1024];
    let mut written = 0;
    while written < sizing.job() {
        let count = (sizing.job() - written).min(block.len() as u64) as usize;
        disk.write_all(&block[..count])
            .expect("the complete protocol job disk must remain writable");
        written += count as u64;
    }
    disk.sync_all().expect("the complete VM write must flush");
    assert_eq!(digest(&base), before);
    let candidate = area.join("images/update");
    let mut update = BudgetFile::create(&candidate, space.clone()).unwrap();
    let before_blocks = candidate.metadata().unwrap().blocks();
    assert!(matches!(
        update.write(&vec![b'U'; 160 * 1024 * 1024]),
        Err(Error::ImageBudget)
    ));
    assert_eq!(candidate.metadata().unwrap().blocks(), before_blocks);
    assert_eq!(candidate.metadata().unwrap().len(), 0);
    assert_eq!(digest(&base), before);
    println!(
        "{}",
        serde_json::json!({"host_bytes":host_bytes,"job_bytes":sizing.job(),"image_budget":sizing.images,"backing_bytes":file.metadata().unwrap().len(),"vm_bytes_written":written,"base_bytes":base.metadata().unwrap().len(),"base_unchanged":true,"budget_refusal_before_allocation":true})
    );
}
