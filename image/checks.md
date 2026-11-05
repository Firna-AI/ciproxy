# Offline image checks

Check the generated QEMU bootstrap without KVM or package downloads:

```sh
python3 image/check-template.py
```

Check the kernel build environment without compiling Linux:

```sh
python3 image/check-kernel-metadata.py
```

This fixture checks neutral metadata at the Make process boundary.
It needs Git, Bash, and Python. It does not prove a real kernel build.

Run the mount-order and metadata regression without KVM:

```sh
sudo python3 image/check-export.py \
  --work-dir .context/ciproxy-mvp/orchestrator-fix-1
```

It needs `chroot`, `ldd`, Bash, Python, tar, zstd, e2fsprogs, `acl`, and
`setcap` from the Linux capability tools. It keeps all fixtures as evidence.
It runs provisioning inside a fixture chroot with stub provisioning tools.
It runs the real export script with stub libguestfs transport boundaries.
Real archive, ext4, and package tools check both outputs and their metadata.
The ext4 check verifies the approved 1% root reserve.
It also checks neutral root labels in the release measurement and manifest.
It checks the compact root's free margin and both `root_disk_bytes` fields
against the actual file length. The fixture uses real ext4 resizing tools.
It grows a copy of the packaged release disk with plain `resize2fs`.
It neither forces resizing nor repairs the copy first.
This check cannot prove real guestfish inspection, mounts, or a VM boot.

## Retained userspace export

To export a retained, complete release tree without software provisioning:

```sh
bash image/export.sh "$PWD/.context/ciproxy-image-build" from-rootfs
```

This uses `WORK_DIR/rootfs` and the retained kernel and software reports.
It rebuilds the root disk and both packages from those inputs.
