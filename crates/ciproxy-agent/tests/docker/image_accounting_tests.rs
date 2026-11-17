//! Attached drive blocks stay outside the physical image and support budget.

use std::fs::{self, File};
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

use crate::fixtures::{TemporaryLayout, vm};
use crate::image_space::{LinuxSpaceUsage, SpaceUsage};
use crate::vm_drives::VmDrive;

#[test]
fn attached_drive_payload_is_not_image_space_but_jail_support_is() {
    for executable in ["firecracker", "renamed-vmm"] {
        let mut temporary = TemporaryLayout::new();
        temporary.layout.firecracker = PathBuf::from(executable);
        let mut vm = vm("image-accounting");
        vm.local_id = uuid::Uuid::new_v4().simple().to_string();
        let root = temporary.layout.jail_root(&vm);
        fs::create_dir_all(&root).unwrap();
        for drive in VmDrive::ALL {
            File::create(drive.path(&temporary.layout, &vm)).unwrap();
        }
        let support = root.join("support.bin");
        File::create(&support).unwrap();
        let usage = LinuxSpaceUsage {
            layout: temporary.layout.clone(),
        };
        let before = usage.used().unwrap();

        for drive in VmDrive::ALL {
            let path = drive.path(&temporary.layout, &vm);
            fs::write(&path, vec![b'D'; 128 * 1024]).unwrap();
            assert!(path.metadata().unwrap().blocks() > 0);
        }
        assert_eq!(usage.used().unwrap(), before, "{executable}");

        fs::write(&support, vec![b'S'; 64 * 1024]).unwrap();
        let charged = support.metadata().unwrap().blocks() * 512;
        assert_eq!(usage.used().unwrap() - before, charged, "{executable}");

        let elsewhere = root.join("support");
        fs::create_dir(&elsewhere).unwrap();
        let named_like_drive = elsewhere.join(VmDrive::DockerJob.file_name());
        File::create(&named_like_drive).unwrap();
        let before = usage.used().unwrap();
        fs::write(&named_like_drive, vec![b'N'; 64 * 1024]).unwrap();
        assert_eq!(
            usage.used().unwrap() - before,
            named_like_drive.metadata().unwrap().blocks() * 512,
            "a drive name outside its jail root must stay charged"
        );
    }
}
