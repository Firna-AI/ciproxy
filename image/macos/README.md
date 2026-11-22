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
The VM disk is sparse, so it takes the space that the guest writes. The
packaging clones the VM files and compresses the disk straight into the
archive, so it needs free space for the archive only. `provision` stops
Packer when the host has less than `MIN_FREE_GB` free, 15 GB by default.
Run from the repository root:

```sh
(
  source ~/ws-i/secrets/r2-xcode-read.env
  GUEST_DIR=/path/to/guest \
    bash image/macos/build.sh "$PWD/.context/macos-image-build"
)
```

Each step below is a phase of `build.sh`: `fetch`, `tools`, `install`,
`setup`, `recovery`, `configure`, `provision`, `facts`, and `package`.
Name phases after the work folder to run only those phases. Set
`GITHUB_API_PAT` to a GitHub token without rights for a higher API rate
limit in the upstream scripts. The token stays out of the image.

1. `fetch.sh` downloads and checks the inputs. Apple serves Xcode only
   after an Apple Account sign-in, so the private R2 bucket `ciproxy-xcode`
   keeps one copy of each archive as `xcode/<XCODE_XIP_NAME>`. When
   `inputs/xcode` has no archive, `fetch.sh` downloads that copy with the
   read-only keys of the [R2 env files](#the-r2-env-files), then checks
   `XCODE_SHA256`. When only some of the keys are set, it names the missing
   ones. `XCODE_DIR` names a local folder with the archive instead,
   and it wins over the bucket. Without either, only the test image can be
   built. The archive stays out of the repository and out of the public
   image. `GUEST_DIR` names the folder that `package-guest.sh` builds: the
   guest, the cache sidecar, the runner patch, and their `install.sh`.
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
   serves the Xcode archive on its loopback address, in place of upstream's
   storage account. A reverse SSH tunnel, which the host opens again after
   each guest restart, brings it to the guest's `127.0.0.1`. So the agent's
   pf rules, which block guests from the host, do not stop the build. Set
   `GITHUB_API_PAT` to a token without rights for a higher GitHub API rate
   limit, or put it in `secrets/github-api-pat` of the work folder. It
   reaches Packer through the environment, not the command line.
   The upstream tests and software report run as upstream.
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
APFS container, and cuts `disk.img` to `DISK_BYTES`. `fix-gpt.py` then
makes the partition map match the new length, because the VM firmware does
not start a guest whose map names sectors after the cut. Then the phase
starts the guest once, which proves that the disk boots.
It changes nothing when the used space does not fit with a 2 GB margin.

The `test-image` phase needs no Xcode archive. It installs the stock runner
and the guest from `GUEST_DIR`, turns on auto-login, reads the guest's
facts, removes the build state, and packages the guest in the release form.
The manifest has `"test_image": true`, and the image ID is `TEST_IMAGE_ID`
of the pins. The image has no Xcode and no upstream toolset, so it is only
for tests. The cleanup removes the build's SSH key, so copy the VM folder
first when you want to run more phases on it.

## Packaging

The VM files go into `artifacts/bundle` as APFS clones. `write-manifest.py`
writes `measurement.json`, `manifest.json`,
and `size-report.txt`. The manifest has the form of the Linux manifest:
`image_id`, `inputs`, `root_disk_bytes`, `guest_sha256`, and
`measurements`. It adds `platform` and `minimum_host_macos`.
`package.sh` writes `macos.tar`, which is not compressed as a whole, with
these members:

- `bundle/disk.img.zst`, `bundle/aux.img`, and `bundle/hardware-model.bin`,
  the files that `ciproxy-vz` reads. The agent writes the machine identifier
  and the MAC address for each VM, so the image has neither.
- `manifest.json`, `measurement.json`, `upstream-software.json`, and
  `software-report.json`.
- `macos-content.sha256`, with the SHA-256 of the extracted `disk.img`, of
  `disk.img.zst`, and of each other member.

`macos_archive.py` writes `macos.tar` with plain regular members, owner 0,
in PAX format. It compresses the disk with zstd straight into the member
`bundle/disk.img.zst`, and fills in that member's fixed-width PAX size at
the end. It writes `macos-content.sha256` from the hashes of the same pass,
and decompresses the disk member once more to compare it with that sum.
macOS `tar` writes sparse files, such as `aux.img`, as GNU sparse members,
which the agent refuses. The script then reads the raw
headers. It fails on a member that is not a regular file, on PAX keys other
than `size`, `path`, and `mtime`, and on a path outside `MEMBERS` in
`crates/ciproxy-agent/src/macos/image_bundle.rs`. It reads that list from
the agent's source, so the two sides cannot drift.

`downloads.json` gives the SHA-256 and byte size of `macos.tar`, and
`SHA256SUMS` covers every artifact. To install, check the size and SHA-256
of `macos.tar`, and extract it. Decompress `bundle/disk.img.zst` to a sparse
`bundle/disk.img`: write each zero block as a hole. Then check
`macos-content.sha256` before the first start. `root_disk_bytes` is the
logical size of `disk.img`, and `free_job_bytes` is the free space inside
it. The agent does not grow the disk. `minimum_host_macos` is the guest's macOS
version. Refuse an older host.

## Publish

`publish.sh` uploads a packaged release image from the build Mac to the
private R2 bucket `ciproxy-macos-images`, under `releases/<image_id>/`.
Run it from the repository root after the `package` phase:

```sh
(
  source ~/ws-i/secrets/r2-publish.env
  bash image/macos/publish.sh "$PWD/.context/macos-image-build/artifacts"
)
```

The script reads `R2_ACCOUNT_ID`, `R2_PUBLISH_ACCESS_KEY_ID`, and
`R2_PUBLISH_SECRET_ACCESS_KEY`. It does these steps in order:

1. It reads `image_id` from `manifest.json`. It refuses an `image_id` that
   is empty or that holds a character other than a letter, a digit, `.`,
   `_`, `+`, or `-`. It refuses a manifest with `"test_image": true`. Give
   `--test-image` before the folder to publish a test image.
2. It checks `SHA256SUMS` in the artifact folder.
3. It stops when `releases/<image_id>/manifest.json` exists. A release
   folder is immutable, so a new build needs a new image ID.
4. It uploads `macos.tar` as a multipart upload with 128 MiB parts. Then it
   uploads `downloads.json`, `SHA256SUMS`, `measurement.json`,
   `software-report.json`, `upstream-software.json`, and `size-report.txt`.
5. It reads the byte size of the uploaded `macos.tar` with a HEAD request.
   It stops when the size differs from `downloads.json`.
6. It uploads `manifest.json` last, with `If-None-Match: *`. So a folder
   with a manifest is complete, and two runs cannot both write it.
7. It prints the value of the GitHub environment variable, for example
   `MACOS_IMAGE_MANIFEST_PATH=releases/<image_id>/manifest.json`.

To ship the release, set `MACOS_IMAGE_MANIFEST_PATH` to that value in the
GitHub environment, first `main-preview`, then `production`. Then deploy
that environment. The control plane offers the release only when that
environment also has `MACOS_IMAGE_URL_PREFIX` and the R2 read key
`R2_IMAGES_KEY`.

The scripts use the AWS CLI v2. `publish.sh` needs version 2.17.34 or
later, the first release with `--if-none-match` on `aws s3api put-object`.
`fetch.sh` warns on a CLI that is not v2 and continues. The scripts set
`AWS_REQUEST_CHECKSUM_CALCULATION` and `AWS_RESPONSE_CHECKSUM_VALIDATION`
to `when_required`, as Cloudflare advises for R2 with AWS CLI 2.23.0 or
later. `r2.sh` gives the R2 keys to each `aws` process in its own
environment only, and it leaves your AWS config and profile unchanged.

### The R2 env files

The R2 keys live only in env files on the build Mac. Each file holds
`export` lines for its two keys and `R2_ACCOUNT_ID`:

- `~/ws-i/secrets/r2-publish.env` holds the token
  `ciproxy-macos-images-publish`: `R2_PUBLISH_ACCESS_KEY_ID` and
  `R2_PUBLISH_SECRET_ACCESS_KEY`. It can read and write
  `ciproxy-macos-images` only.
- `~/ws-i/secrets/r2-xcode-read.env` holds the token `ciproxy-xcode-read`:
  `R2_XCODE_ACCESS_KEY_ID` and `R2_XCODE_SECRET_ACCESS_KEY`. It can read
  `ciproxy-xcode` only.

Source the publish file before `publish.sh`. Source the Xcode file before
`fetch.sh` or `build.sh`. Source each file in a subshell, as the commands
above do, so that the later commands of your shell do not get the keys.
The scripts remove the export flag of the keys, so that only `aws` gets
them. Of the phases of `build.sh`, only `fetch` gets the Xcode keys. Keep
each file at mode 600 in a folder with mode 700. Keep the files out of the
repository. Never print a file, and do not source one in a shell with
`set -x`. A `bash -x` trace of `publish.sh` or `fetch.sh` shows no key.

## Checks

`python3 image/macos/check-template.py` checks the template and toolset
preparation offline. `python3 image/macos/check-archive.py` checks the
archive writer and checker against the agent's member list.
`python3 image/macos/check-r2.py` runs `publish.sh` and the Xcode download
of `fetch.sh` against a fake `aws` on `PATH`. It checks the upload order,
the refusals, and that no key reaches the output, a trace, or the `aws`
arguments. It needs bash and `shasum`, and runs on Linux too. Lint the R2
scripts from the repository root:

```sh
shellcheck image/macos/publish.sh image/macos/fetch.sh image/macos/r2.sh
for script in publish fetch r2; do bash -n "image/macos/$script.sh"; done
```

`image/macos/.shellcheckrc` makes shellcheck follow the sourced files.
