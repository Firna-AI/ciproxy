# The macOS job image

This Apache-2.0 image definition builds the macOS job image for own Macs.
It keeps the software recipe of the GitHub runner image that `macos-latest`
names on arm64, so a workflow keeps its YAML apart from the `runs-on` line.
The agent runs each macOS job in a fresh clone of this image, in a VM over
Apple's Virtualization.framework. The [Linux image](../README.md) is the
model for the pins, the manifest, and the checksums.

## Pins

[pins.env](pins.env) fixes each input by version and SHA-256.

- The GitHub image is `macos-26-arm64/20261117.0516`, at commit
  `6ae22b7fe1e9897ad8220e69f154a603cbd4b0ab` of `actions/runner-images`.
  Its software inventory `internal.macos-26-arm64.json` is the reference.
  It runs macOS 26.7.2 (25G332) with Xcode 26.6 (17F113) as the default.
  The image pins `MACOS_TARGET`, macOS 26.7.3 (25G342), because Software
  Update offers no older 26.x update. It is one security update newer.
- The restore image is macOS 26.6.2 (25G83). Apple publishes no newer full
  restore image for macOS 26, so the build updates the guest to the pinned
  26.7.3 with Software Update. The build stops when Software Update does not
  offer that exact build.
- The runner is the stock runner 2.338.2 for `osx-arm64`.
- `XCODE_LINKS` selects the Xcode versions of the upstream toolset that the
  image keeps. The pin keeps only the default Xcode 26.6. When the image
  keeps no version with its own simulator runtimes, the default Xcode
  installs its runtimes. The upstream toolset has 7 versions and about 20
  runtimes, which need about 250 GB.
- Packer 1.16.2 runs the upstream template.

Virtualization.framework runs no guest that is newer than its host. So a
customer Mac needs the guest's macOS version or later. The manifest names it
in `minimum_host_macos`.

Homebrew packages of the recipe resolve at build time, as upstream. The
build keeps the generated software report next to the reference inventory.
Compare them before each release.

## The build

The build runs on an Apple Silicon Mac with the host macOS at the guest's
version or later. It needs the repository's Rust toolchain, `zstd`, `curl`,
Python 3, and the Xcode command line tools, which supply `swiftc` for the
screen reader. The VM tool is the crate
[ciproxy-image-vm](../../crates/ciproxy-image-vm), over `ciproxy-vz`.
The VM disk is sparse. Keep free space for the VM disk, the compact copy,
and the archive. Run from the repository root:

```sh
XCODE_DIR=/path/to/xcode GUEST_DIR=/path/to/guest \
  bash image/macos/build.sh "$PWD/.context/macos-image-build"
```

Each step below is a phase of `build.sh`: `fetch`, `tools`, `install`,
`setup`, `recovery`, `configure`, `provision`, `facts`, and `package`.
Name phases after the work folder to run only those phases. Set
`GITHUB_API_PAT` to a GitHub token without rights for a higher API rate
limit in the upstream scripts. The token stays out of the image.

1. `fetch.sh` downloads and checks the inputs. Apple serves Xcode only
   after an Apple Account sign-in, so a person downloads the archive once
   and names its folder in `XCODE_DIR`. The archive stays out of the
   repository and out of the public image. `GUEST_DIR` names the folder that
   `package-guest.sh` builds: the guest, the cache sidecar, the runner patch,
   and their `install.sh`.
2. The VM tool installs macOS from the restore image onto a sparse disk of
   `DISK_BYTES`.
3. The Setup Assistant runs over VNC. `setup-assistant.py` reads each
   screen with Apple's Vision text reader (`ocr.swift`). Each step waits for
   the text of its screen, then clicks a control by its label or types.
   An unexpected screen stops the build and keeps a screenshot. The steps
   create the account `runner`, as on GitHub, and skip the Apple Account,
   location, analytics, Screen Time, and FileVault. A random password for
   each build stays in the work folder.
4. In Terminal, the steps add a sudoers file without a password and start
   Remote Login. The rest runs over SSH.
5. The guest starts once in macOS Recovery, and the steps run
   `csrutil disable`. GitHub's VMs run without System Integrity Protection,
   and the upstream recipe writes the system TCC database.
6. `configure-guest.sh` sets the time zone `GMT`, because macOS refuses
   `UTC`, and turns off sleep. `update-macos.sh` installs the pinned macOS
   update.
7. `prepare-template.py` copies the upstream template and keeps every
   upstream provisioner. It removes the Anka plugin and its source, selects
   the upstream `null` builder over SSH, and writes the trimmed toolset.
   It appends three ciproxy steps: `install-runner.sh` puts the runner in
   `/opt/actions-runner`, the guest's `install.sh` adds the guest and its
   launchd service, and `clean-image.sh` removes the build state. The host
   serves the Xcode archive to the guest over HTTP, in place of upstream's
   storage account. The upstream tests and software report run as upstream.
8. `guest-facts.sh` reads the macOS version, the used and free bytes of the
   data volume, SIP, and Xcode. Then the guest turns off, and the build
   packages it.

The shipped image has no SSH key and accepts no SSH password. The auto-login
file holds the password in a reversible form.

## The test image

`build.sh WORK_DIR test-image` packs a test image for the end-to-end test.
Run it after the `configure` phase. For a small test disk, run the opt-in
`shrink` phase first, with `DISK_BYTES` set in the environment. It removes
the update's leftovers and the guest's Recovery container, shrinks the main
APFS container, and cuts `disk.img` to `DISK_BYTES`. Then it starts the
guest once, which proves that the disk boots, and repairs the partition map.
It changes nothing when the used space does not fit with a 2 GB margin. It needs no Xcode archive. It installs
the stock runner and the guest from `GUEST_DIR`, turns on auto-login, reads
the guest's facts, removes the build state, and packages the guest in the
release form. The manifest has `"test_image": true`, and the image ID is
`TEST_IMAGE_ID` of the pins. The image has no Xcode and no upstream toolset, so it is only
for tests. The cleanup removes the build's SSH key, so copy the VM folder
first when you want to run more phases on it.

## Packaging

`compact.py` copies the VM disk to a sparse file. Holes and zero blocks stay
unallocated. `write-manifest.py` writes `measurement.json`, `manifest.json`,
and `size-report.txt`. The manifest has the form of the Linux manifest:
`image_id`, `inputs`, `root_disk_bytes`, `guest_sha256`, and
`measurements`. It adds `platform` and `minimum_host_macos`.
`package.sh` compresses the disk with zstd into `bundle/disk.img.zst`. It
writes `macos.tar`, which is not compressed as a whole, with these members:

- `bundle/disk.img.zst`, `bundle/aux.img`, and `bundle/hardware-model.bin`,
  the files that `ciproxy-vz` reads. The agent writes the machine identifier
  and the MAC address for each VM, so the image has neither.
- `manifest.json`, `measurement.json`, `upstream-software.json`, and
  `software-report.json`.
- `macos-content.sha256`, with the SHA-256 of the extracted `disk.img`, of
  `disk.img.zst`, and of each other member.

`downloads.json` gives the SHA-256 and byte size of `macos.tar`, and
`SHA256SUMS` covers every artifact. To install, check the size and SHA-256
of `macos.tar`, and extract it. Decompress `bundle/disk.img.zst` to a sparse
`bundle/disk.img`: write each zero block as a hole. Then check
`macos-content.sha256` before the first start. `root_disk_bytes` is the
logical size of `disk.img`, and `free_job_bytes` is the free space inside
it. The agent does not grow the disk. `minimum_host_macos` is the guest's macOS
version. Refuse an older host.

## Checks

`python3 image/macos/check-template.py` checks the template and toolset
preparation offline.
