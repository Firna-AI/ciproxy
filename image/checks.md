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

## Kernel build metadata

Kernel builds use `builder@localhost` and a fixed build version.
The timestamp uses the source commit time through
`SOURCE_DATE_EPOCH`. Source archives can supply that value explicitly.
Without source metadata or an explicit value, the timestamp uses the Unix epoch.

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
It checks the small root's free margin and both `root_disk_bytes` fields
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

## Release growth proof

The measured root disk is 60,263,759,872 bytes. The base margin is 64 MiB.
The producer measures empty ext4 metadata and keeps the 1% root reserve.
The agent adds its filesystem allowance for ext4 metadata and reserves.

The proof calls the real `ciproxy_agent::root_sizing::RootSizing` from
commit `d6d18e6dca141fc822232830109ee34c29a3ce95` of the agent sizing branch.
That branch is not merged into the image branch.
Each row uses a fresh sparse copy and the helper's returned length, `R`.
Plain `resize2fs` 1.46.5 grows the copy without `-f` or a prior repair.
A read-only Firecracker probe then runs `df` as runner UID 1001.

| Requested free bytes | Helper `R`, bytes | Measured free bytes |
| ---: | ---: | ---: |
| 1,000,000,000 | 61,311,987,712 | 1,557,065,728 |
| 20,000,000,000 | 80,899,612,672 | 20,641,435,648 |
| 60,000,000,000 | 122,136,727,552 | 60,811,378,688 |
| 500,000,000,000 | 575,744,974,848 | 502,754,795,520 |

The default job leaves 1,600,387,328 bytes of the 82,500,000,000-byte job
area on the 250 GB minimum machine. This exceeds the required 1 GB spare.
The source disk stays clean and unchanged during these copy-based proofs.
These probes measure free space before jobs write data.
Registered-job and full service checks remain separate release work.
