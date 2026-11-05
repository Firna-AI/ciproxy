# Offline image checks

Check the generated QEMU bootstrap without KVM or package downloads:

```sh
python3 image/check-template.py
```

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
This check cannot prove real guestfish inspection, mounts, or a VM boot.
