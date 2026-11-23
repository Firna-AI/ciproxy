# ciproxy-image-vm

The VM tool of the macOS image build. [image/macos](../../image/macos)
drives it from `build.sh`. This crate has the Apache-2.0 licence in
[LICENSE](LICENSE).

## Responsibilities

Install macOS into a new bundle, run a VM with a VNC server, and print the
guest's address, over [ciproxy-vz](../ciproxy-vz). Hold no build steps.
The scripts of the image build hold them.

## What This Crate Does

- `install --restore-image IPSW --bundle DIR --disk-bytes N` creates the
  bundle and installs macOS. The Setup Assistant runs on the first start.
- `run --bundle DIR --vnc-password-file F --status-file F --stop-file F`
  starts the VM, with `--recovery` in macOS Recovery. It serves the display
  over VNC on a free port with the password, and writes the port and the MAC
  address to the status file. It runs until the guest turns off, or until
  the stop file appears.
- `ip --bundle DIR --wait-seconds N` prints the guest's IPv4 address from
  the DHCP leases.

The VNC server of Virtualization.framework listens on every interface, so
each build uses a random password. The binary needs the
`com.apple.security.virtualization` entitlement. `build.sh` signs it with
[vz.entitlements](../ciproxy-vz/vz.entitlements). On other systems, the
binary only prints that it runs on macOS.
