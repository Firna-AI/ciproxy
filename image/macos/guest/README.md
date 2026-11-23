# macOS guest

This folder installs the [ciproxy guest](../../../crates/ciproxy-guest/README.md)
into the macOS job image. The guest runs one stock GitHub runner in a fresh
macOS VM on a customer's own Mac. The files have the Apache-2.0 licence in
[LICENSE](../../LICENSE).

## Files

- [install.sh](install.sh) installs the guest into the image. Run it as
  root, as the last step of the image build before its cleanup.
- [dev.ciproxy.guest.plist](dev.ciproxy.guest.plist) is the launchd daemon.
  It starts the guest once at boot, as root. It replaces the systemd unit
  `ciproxy-guest.service` of the Linux image.
- [start.sh](start.sh) reads `/etc/ciproxy/guest.env`, starts the guest,
  and powers the VM off after the guest ends, also after a failed start.
- [runner-entry.sh](runner-entry.sh) loads the login environment of the
  runner account and starts the stock listener with `--once`.

## Build the folder

Run on an Apple Silicon Mac with the pinned Rust toolchain and the .NET
SDK of the pinned runner source:

```sh
image/macos/package-guest.sh OUT_DIR
```

The script builds `ciproxy-guest` and `ciproxy-cache-sidecar` for
`aarch64-apple-darwin`. It builds the patched Worker for `osx-arm64` with
`image/build-runner.sh WORK_DIR osx-arm64`. The runner source takes its OS
constants from the build host, so this Worker builds only on macOS. It
downloads the pinned stock runner `osx-arm64` 2.338.2 and checks its
SHA-256. It makes the
[Worker overlay](../../runner-patch/README.md#worker-overlay) from the
Worker and that runner into `OUT_DIR/worker`. The script copies the
binaries, `install-worker.sh`, and the files of this folder into `OUT_DIR`.
Its work files stay under `.context/ciproxy-macos-guest/`.

## Install into the image

Before the install, the image needs:

- the stock runner `osx-arm64` 2.338.2 in `/opt/actions-runner`;
- the account `runner`, home `/Users/runner`, with passwordless `sudo`;
- automatic login of `runner`, as on GitHub's macOS runners.

Copy `OUT_DIR` into the VM, then run:

```sh
sudo ./install.sh IMAGE_ID
```

The script installs the two binaries in `/usr/local/bin`, the start script
and the runner entry in `/usr/local/libexec`, and the daemon in
`/Library/LaunchDaemons`. It installs the Worker overlay with
`install-worker.sh`. That script keeps the stock `Runner.Worker.dll` as
`Runner.Worker.stock.dll` beside the patched `Runner.Worker.own.dll`. It
also installs the raised package assemblies and the changed deps file, and
it stops when the runner is not the pinned stock runner. The install script
writes `IMAGE_ID`, the guest port 5252, and the account in
`/etc/ciproxy/guest.env`. It does not start the guest. The guest starts at
the next boot. `CIPROXY_RUNNER_ACCOUNT` names another account, for a test.
No registration or token is in the folder or the image.

## How a job runs

1. The VM boots, and launchd starts the guest as root.
2. The guest listens on vsock port 5252. The agent connects through the
   virtio socket device of Virtualization.framework and sends the JIT
   registration in `ConfigureRunner`. The registration stays in memory.
3. `launchctl asuser` starts the runner entry in the launchd domain of
   `runner`, and `sudo` changes the account. The entry gets only the JIT
   registration and the assignment socket path in its environment.
4. Before the first step, the patched Worker sends the cache handoff over
   `/var/run/ciproxy/assignment.sock`, as
   [Cache handoff](../../../docs/protocol/cache-handoff.md) says. The host
   decides, and the steps then use the proxy on `127.0.0.1:8230` or GitHub.
5. When the runner ends, the guest reports `RunnerExited` and powers the VM
   off with `shutdown -h now`.

The guest log is `/var/log/ciproxy-guest.log` in the VM. It holds no
registration or token.

## Differences from the Linux image

- No Docker: the guest reports no Docker bytes and attaches no Docker disk.
  The proxy listens on loopback, because the steps run in the VM itself.
- A power-off ends the VM. On Linux, a reset ends the Firecracker process.
- `launchctl asuser` and `sudo` replace `runuser`.
- The socket folder is `/var/run/ciproxy`, because macOS has no `/run`.
