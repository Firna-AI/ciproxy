# Linux arm64 jobs on own machines

Status: Active. On branch `calummoore/arm-linux-support-ojmg`, no pull
request yet. The plan started on `calummoore/ci-proxy-on-raspberry-pi`.
The pull request that completes the work deletes this plan, as the `plans`
skill says.

ciproxy runs the GitHub Actions jobs of a customer on the customer's own
machines first, and on Boat sandboxes when those machines are busy. Today an
own machine is an x64 Linux computer or an Apple Silicon Mac. This plan adds
a third platform, Linux arm64. A job whose `runs-on` names an arm64 label
runs in a fresh Firecracker VM on one of the customer's arm64 Linux machines.
The goal is the Raspberry Pi. The first target is any arm64 Linux machine
with KVM, and a Raspberry Pi 5 is the first tested model.

The plan keeps the slot rule, so a machine with 8 GB of memory still gets no
slots. The follow-up item
[Small labels do not fit small machines](follow-ups/small-label-fit.md)
holds that work. The plan adds no Linux arm64 jobs on Macs. Boat offers
x86_64 machines only. Decision 3 says where an arm64 job runs when the arm64
machines are busy.

Contract owners:

- The [architecture doc](../docs/architecture.md).
- The protocol docs [agent](../docs/protocol/agent.md),
  [agent install](../docs/protocol/agent-install.md),
  [agent images](../docs/protocol/agent-images.md),
  [labels](../docs/protocol/labels.md),
  [job routing](../docs/protocol/job-routing.md),
  [overflow](../docs/protocol/overflow.md),
  [billing](../docs/protocol/billing.md), and
  [security](../docs/protocol/security.md).
- The READMEs of [the agent](../crates/ciproxy-agent/README.md),
  [the protocol crate](../crates/ciproxy-protocol/README.md),
  [the image](../image/README.md), and [the gate](../xtask/README.md), and
  the [release server guide](../deploy/release-server.md).

## Terms

- **Platform:** the operating system and the processor type of a machine, a
  label, an image, and a job VM. This plan adds Linux arm64 to Linux x64 and
  macOS arm64.
- **arm64 machine:** an own machine with a 64-bit Arm processor that runs
  Linux and the agent. The product calls it a machine.
- **arm64 label:** a label whose jobs run in a Linux arm64 job VM.
- **Slot:** the unit of machine capacity: 2 hardware threads and 8 GB of
  memory, after the host keeps a quarter of each.
- **Spike:** a short test that proves an approach before the build. Its code
  stays under `.context/linux-arm64-own-machines/spikes/`.

## Decisions

User decisions, 22 November 2026:

- Support the Raspberry Pi as an own machine. The user wrote "yes, lets
  support it" after the answer that ciproxy does not run on a Pi.
- Target any arm64 Linux machine first. The user provides a Pi later. The
  user wrote "first we can just target an ARM linux".
- Keep the slot rule for now. The user wrote that customers "will be able to
  specify smaller options" and "lets not worry about the sizing issue for
  now". A label of any size still takes a whole slot, so the follow-up item
  above records the change.

User decisions, 23 November 2026:

- Decision 5 = D: an arm64 Linux VM with nested virtualization on the Mac
  mini M4, over Tailscale, runs the VM checks, the image build, and the
  live checks before the Pi arrives. The user wrote "5 = D" and "I will
  get the pi ready in the background".

Earlier user decisions that hold:

- One image for each platform until launch, from 5 November 2026.
- Macs run only macOS jobs, from 21 November 2026. Linux arm64 jobs on Macs
  were "rejected for now".
- A Mac counts as one own machine at the price of any own machine, from
  21 November 2026.

Design calls, mine, 22 November 2026:

- The platform `linux_arm64` covers every arm64 Linux machine with KVM.
  Reason: the same code serves each one. The docs name each tested model.
- Decision 1 chooses the VM layer. Cloud Hypervisor is out, because it
  needs a newer interrupt controller than the Pi 4 and the Pi 5 have.
- An arm64 machine counts as one own machine at the same price, as a Mac
  does.
- The default arm64 labels mirror the two smaller Linux labels:
  `ciproxy-arm64` with 2 vCPU, 8 GB, and 20 GB, and `ciproxy-arm64-medium`
  with 4 vCPU, 16 GB, and 20 GB. This is a proposal, as for macOS.
  `ciproxy-arm64-medium` takes 2 slots, so it fits no Raspberry Pi.
- `ciproxy-arm64` is the warm-runner label and the default machine label of
  the platform, as `ciproxy-macos` is for Macs. The customer can delete it.
  The arm64 machines then keep no warm runners.
- arm64 machines use the Linux distributions of x64 machines. 64-bit
  Raspberry Pi OS takes the Debian path. 32-bit systems stay out, because
  32-bit Linux has no KVM host and GitHub retires its 32-bit runner.
- The docs advise an SSD, not a microSD card. The agent adds no new check.
- Releases add `aarch64-unknown-linux-gnu` archives of the agent and the
  cache sidecar, built with `cargo zigbuild` for glibc 2.17, as on x64. The
  upstream installer and updater already map Linux `aarch64` to that
  target.
- The arm64 image gets its own image budget and disk minimum, measured from
  the image. Until then the disk minimum is a placeholder, as for the Mac.
- The Pi checks measure Firecracker on the 16K-page and the 4K-page kernels
  of the Pi 5. The plan then proposes a kernel rule as a decision, when one
  is needed.
- Work that needs no arm64 hardware goes first.

### 1. No approach chosen for jobs on a Pi

- **Severity:** High. **Effort:** Large.
- **Source:** Design.
- **Category:** Docs or spec.
- **Location:** `docs/protocol/agent-images.md:28`
- **Background:**
  - The Linux image has the software of GitHub's x64 `ubuntu-latest` image,
    so a workflow keeps its YAML apart from its `runs-on` line.
  - Since May 2026, `actions/runner-images` holds the open Packer recipe of
    GitHub's arm64 Ubuntu images. ciproxy builds its x64 image from that
    project with its own QEMU build.
  - The arm64 image lacks the Android SDK, Haskell, PyPy, Julia, Miniconda,
    Bicep, CodeQL, and the Chrome and Edge browsers of the x64 image. In
    November 2025 it used 27 GB of disk at job start, against 50 GB on x64.
  - A user decision of 5 November 2026 allows one image for each platform
    until launch.
  - All options assume a 64-bit system, because 32-bit Linux has no KVM
    host.
- **Problem:** Nothing decides which software arm64 jobs find, or whether a
  job may use the devices of its machine.
- **Impact if no action:** The image milestone cannot start, and the docs
  cannot state the disk minimum.
- **Root question:** What does an arm64 machine give the jobs that run on
  it?
- **Options:**
  - **A.** Fresh Firecracker VMs with a small image: Ubuntu, Docker, the
    runner, and the guest. It fits small disks and downloads fast. It can
    build with no KVM, for example on a GitHub-hosted arm64 runner. Jobs
    install their own tools, so workflows differ from GitHub's arm64
    runners. It adds an arm64 check to the gate and arm64 tests to CI.
    Effort: Large.
  - **B.** Fresh Firecracker VMs with a full image from GitHub's open arm64
    recipe, for the Ubuntu release of the x64 image. Workflows written for
    GitHub's arm64 runner of that release work unchanged. It probably needs
    a 256 GB disk, and its build needs an arm64 host, as decision 5 says.
    It adds the same gate check and CI tests. Effort: Large.
  - **C.** Jobs run on the machine itself, with no VM, so they can use its
    GPIO pins, camera, and USB devices. It drops the fresh-VM rule of job
    isolation, so it suits trusted repositories only. Effort: Large.
  - **D.** Fresh QEMU VMs over KVM, with the image of B, in place of
    Firecracker. More reports show QEMU with KVM on the Pi 4 and the Pi 5
    than Firecracker. QEMU can also give a USB device to a job. It adds a
    second VM layer to the agent for one platform, with its own jail and a
    larger device model. Effort: Large.
- **Left out:**
  - x64 emulation on the Pi, because emulated x64 code runs many times
    slower.
  - A full and a small arm64 image side by side, because of the user
    decision of 5 November 2026.
  - Linux arm64 jobs on Macs too, because they are outside the goal, and
    the user kept Macs on macOS jobs on 21 November 2026.
  - Support for the Pi only, because the user chose to target any arm64
    Linux machine first.
  - A fresh container for each job, because it breaks the fresh-VM rule of
    job isolation, as option C does, and gives no device access.
  - Cloud Hypervisor, because it needs the newer interrupt controller.
- **Wider fix:** None, because this is a design choice, not a defect.
- **Previous examples:** The macOS image decision of 21 November 2026 in
  `docs/protocol/agent-images.md`: the image has the software of GitHub's
  `macos-latest` image.
- **Recommendation:** B. Workflows that run on GitHub's arm64 runners then
  run unchanged, as with ciproxy's x64 and macOS images. D stays the
  fallback if Firecracker fails the Pi checks.
- **Action:** ask the user, because it adds a platform image and a gate
  check, and no recorded decision covers either.

### 3. Boat cannot run arm64 overflow jobs

- **Severity:** Medium. **Effort:** Small.
- **Source:** Design.
- **Category:** Docs or spec.
- **Location:** `docs/protocol/overflow.md:90`
- **Background:**
  - Today the jobs of every Linux label can overflow to Boat. A macOS job
    never does. It waits for a free Mac.
  - Boat's docs say "All sizes are x86_64."
  - GitHub gives a queued job only to the runners of its label. So ciproxy
    cannot pass an arm64 job to GitHub's own arm64 runners.
  - Six places in the Rust code send only `linux_x64` labels to Boat:
    `crates/ciproxy-api/src/handlers/label_views.rs:20`,
    `crates/ciproxy-api/src/handlers/label_boat.rs:48`,
    `crates/ciproxy-github/src/app/installation.rs:212`,
    `crates/ciproxy-scaler/src/snapshot.rs:74`, and two store queries in
    `crates/ciproxy-store-pg/src/overflow/authority.rs:32` and
    `crates/ciproxy-store-pg/src/boat_runtime.rs:94`.
  - The dashboard tests for a macOS label instead, for example in
    `ts/app/src/view/settings-label-form.ts:114`. So it would offer Boat
    on an arm64 label, and the API would refuse the save.
- **Problem:** The protocol lets every Linux label overflow to Boat. arm64
  labels are Linux labels, and Boat cannot run them.
- **Impact if no action:** The dashboard shows a Boat switch on arm64
  labels, and the API refuses each label that turns it on.
- **Root question:** Where does a job run when its own machines are busy
  and no backend offers its platform?
- **Options:**
  - **A.** It waits for a free arm64 machine, as a macOS job waits for a
    Mac. A follow-up item adds Boat when Boat offers arm64. Effort: Small.
  - **B.** A second overflow backend with arm64 VMs billed by the second,
    such as AWS EC2 Graviton. It needs a new account flow and an arm64
    snapshot build. An earlier decision rejected EC2 as the first backend.
    Effort: Large.
  - **C.** One rule: a label uses a backend only when the backend offers the
    platform of the label. The backend contract names the platforms of each
    backend, and Boat offers Linux x64 now. One exhaustive match in
    `crates/ciproxy-protocol` replaces the six Rust tests. The API tells the
    dashboard which platforms Boat serves, so the dashboard drops its own
    macOS tests. arm64 jobs then wait, as in A. Effort: Medium.
  - **D.** C, and Namespace as a second backend for arm64 jobs. Its Linux
    instances on Apple Silicon M4 and later give `/dev/kvm` on arm64. They
    start by API and bill per minute. They are in early access, need the
    Team plan, and cost 7 times its other arm64 instances. Firecracker does
    not test this host type. It needs a new account flow, as Boat has.
    Effort: Large.
- **Left out:**
  - GitHub's hosted arm64 runners as overflow, because GitHub gives the job
    only to the runners of its label.
  - The other 40 sandbox providers of the survey of 23 November 2026,
    because none documents `/dev/kvm` on arm64. Blacksmith and AWS
    AgentCore document that they have none. The notes are in
    `.context/linux-arm64-own-machines/arm64-kvm-providers.md`.
- **Wider fix:** C, because the next platform or a Boat arm64 offer then
  needs no new search through the code.
- **Previous examples:** The rule that macOS jobs never overflow, in
  `docs/protocol/overflow.md`.
- **Recommendation:** C. It gives arm64 jobs the behaviour of A and puts the
  rule in one place. The first recommendation was A.
- **Action:** ask the user, because options A and C change the rule that
  every Linux label can overflow to Boat.

### 5. No arm64 machine until the Pi arrives

- **Severity:** High. **Effort:** Medium.
- **Source:** Design.
- **Category:** Process.
- **Location:** None.
- **Background:**
  - On 22 November 2026 the user said that a Pi comes later.
  - The arm64 VM code of the agent, the image build, and the live checks
    need an arm64 machine with KVM.
  - GitHub's hosted arm64 runners have no KVM, and Boat has no arm64
    machines. Common cloud arm64 VMs do not expose KVM either.
  - Firecracker tests its releases on AWS Graviton bare-metal servers.
  - QEMU 11 for arm64 installs in the workspace. In software emulation, it
    can give an emulated arm64 host its own KVM for nested VMs. This is
    slow.
  - `image/README.md` rebuilds the image for each runner release and each
    security update. So the build host is a lasting need, and a Pi with 4
    cores builds the full recipe slowly.
  - On a Mac with an M3 chip or later and macOS 15 or later, Apple's
    Virtualization.framework can give a Linux VM nested virtualization. The
    Mac mini M4 of the macOS work qualifies.
- **Problem:** Until the Pi arrives, no arm64 machine with KVM is available.
- **Impact if no action:** The arm64 VM code of the agent waits untested
  until the Pi arrives, and so do the image build and the live checks.
- **Root question:** Which arm64 hosts with KVM run the VM checks and build
  the arm64 image of each release, before and after the Pi arrives?
- **Options:**
  - **A.** Nowhere until the Pi arrives. The arm64 VM checks, the image
    build, and the live checks then run on the Pi. Effort: Small.
  - **B.** An emulated arm64 host in the workspace checks the VM setup of
    the agent and a boot to the guest now. The image build and the live
    checks wait for the Pi. Effort: Medium.
  - **C.** A rented arm64 bare-metal server with KVM runs the image build
    and the live checks now. Examples are a GCP C4A bare-metal instance
    with Google Axion processors, an AWS Graviton `.metal` instance, and a
    Hetzner RX server. ciproxy already deploys to the GCP project
    `ciproxy`. The smallest C4A bare-metal type has 96 vCPUs, so it costs
    much more per hour than a small server. Effort: Medium.
  - **D.** An arm64 Linux VM with nested virtualization on your Mac mini
    M4, over Tailscale, as the macOS work did. It runs the VM checks, the
    image build, and the live checks now, at no new cost. A short check
    must first show `/dev/kvm` and a Firecracker boot in that VM. The VM
    has the newer interrupt controller, so the Pi's own path still waits
    for the Pi. The VM shares the memory and disk of the Mac with the macOS
    work. Effort: Medium.
  - **E.** Build the arm64 image in Azure with the upstream `azure-arm`
    source, as GitHub does, and convert its disk into the Firecracker root.
    It needs no KVM host for the build, but the VM checks still need
    another option. It needs an Azure subscription and a new conversion
    step, and each build costs money. Effort: Large.
  - **F.** A Namespace Linux instance on Apple Silicon, started by API from
    CI or by hand. It gives `/dev/kvm` on arm64 with no hardware of yours.
    It bills per minute at 7 times its other arm64 instances, in early
    access on the Team plan. An instance lives 3 to 24 hours by plan.
    Firecracker does not test this host type, so a short check comes
    first. Effort: Medium.
- **Left out:**
  - GitHub's hosted arm64 runners and common cloud arm64 VMs, because they
    have no KVM. EC2 nested virtualization covers Intel types only. GCP
    nested virtualization excludes VMs with Arm processors.
  - The other 40 sandbox providers of the survey of 23 November 2026,
    because none documents `/dev/kvm` on arm64.
  - Firecracker Nitro Sandboxes on ordinary Graviton EC2 instances, from
    Firecracker v1.18.0. They need no KVM, but they do not check the KVM
    path of the agent and the Pi. They need a second VM backend in the
    agent.
- **Wider fix:** None, because this is hardware, not a defect.
- **Previous examples:** The macOS work used the user's Mac mini over
  Tailscale, in `plans/macos-own-machines.md` on the branch
  `calummoore/support-mac-os-kq1u`.
- **Recommendation:** D. It runs every check and the image build on real
  hardware virtualization now, at no new cost. If its short check fails,
  use B. The first recommendation was B. F uses the same Apple nested
  virtualization as D, so it suits builds when the Mac is off, not a
  failed check.
- **Action:** ask the user, because options C, D, E, and F need your
  hardware, accounts, or money.
- **Answer:** D, from the user on 23 November 2026. If the short check of D
  fails, the next host is a new decision for the user.
- **Short check:** passed on 23 November 2026. A Lima 2.3.0 Ubuntu 24.04
  arm64 VM on the Mac mini M4 has `/dev/kvm`. Firecracker v1.17.0 aarch64
  boots a guest to its login prompt in about 8 seconds there. The agent's
  own VM setup waits for Milestone 5. The steps are in
  `.context/linux-arm64-own-machines/spikes/README.md`.

## Milestone 1: Mockups

Tags: mockup

Depends on: none

Show Linux arm64 labels and arm64 machines on each screen that shows a
platform, as the macOS mockups do.

- [ ] Name the platforms so that a reader can tell Linux x64 from Linux
      arm64, on each screen that names a platform.
- [ ] Add Linux arm64 to the platform choice of the label dialog. Its Boat
      switch and "When nothing fits" follow decision 3.
- [ ] Show the two default arm64 labels in the labels list of Settings.
- [ ] Show an arm64 machine, a Raspberry Pi 5, on the Machines page and the
      machine page, with its platform, processor, and slots.
- [ ] Show an arm64 machine with 8 GB and no slots, with the reason.
- [ ] Add arm64 Linux and the Raspberry Pi to the add-machine guide and the
      labels reference of the docs mockups.
- [ ] Run `npm --prefix ts run mockups:check`. Smoke-test the changed pages.
- [ ] Update the design and the TODOs of this plan for each problem that the
      mockups show.

## Milestone 2: Documentation and protocol

Depends on: 1

State the requirements and the decisions of the Linux arm64 platform in the
protocol docs and the architecture doc.

- [ ] `docs/protocol/agent.md`: the Linux arm64 platform, its Firecracker
      job VMs, and its virtualization check.
- [ ] `docs/protocol/agent-install.md`: what an arm64 machine needs, the
      placeholder disk minimum, and the SSD advice.
- [ ] `docs/protocol/agent-images.md`: the arm64 machine image, as decision
      1 says, and no Boat snapshot for arm64.
- [ ] `docs/protocol/labels.md`: the Linux arm64 platform, the two default
      arm64 labels, and the Boat rule of decision 3.
- [ ] `docs/protocol/job-routing.md` and `docs/protocol/overflow.md`: where
      an arm64 job runs, as decision 3 says.
- [ ] `docs/protocol/billing.md`: arm64 machines in the tier features, at
      the price of any own machine.
- [ ] `docs/protocol/security.md`: the job isolation of arm64 job VMs, the
      same as on x64.
- [ ] `docs/architecture.md`: the third platform.
- [ ] Run `cargo xtask check --include markdown`.

## Milestone 3: The platform in the protocol crate

Depends on: 2

- [ ] Add `Platform::LinuxArm64`, `linux_arm64` on the wire, with no VM
      limit, in `crates/ciproxy-protocol`. Update its README and the wire
      snapshots.
- [ ] Run the tests of the protocol crate and the early check.

## Milestone 4: The platform in the control plane

Depends on: 3

- [ ] Add a migration that allows `linux_arm64` in the platform checks, and
      its text mapping in `crates/ciproxy-store-pg`, with store tests.
- [ ] Carry the platform through the API, the scaler, and the gateway.
      Regenerate the API types with `npm --prefix ts run api:generate`.
- [ ] Add the default arm64 labels to the installation defaults in
      `crates/ciproxy-github`, with the Boat rule of decision 3.
- [ ] Give Linux arm64 its warm-runner label and default machine label,
      `ciproxy-arm64`, in `crates/ciproxy-scaler/src/snapshot.rs` and the
      API. Replace the hand-made platform lists, such as `warm_candidate`
      in `crates/ciproxy-scaler/src/state.rs`, with exhaustive matches.
- [ ] Apply the Boat rule of decision 3 at each place that tests for
      `linux_x64` today, in the API, the GitHub defaults, the scaler, and
      the two store queries. Test that arm64 labels and runners follow it.
- [ ] Serve the arm64 image release, so that an arm64 machine gets only
      an arm64 image, and waits while none exists. Add a `linux_arm64`
      arm to the platform match of `SignedReleases::current` in
      `crates/ciproxy-control-plane/src/image_release.rs`, beside the
      `linux_x64` and `macos_arm64` arms.
- [ ] Run the targeted tests, with the store tests, and the early check.

## Milestone 5: The agent on Linux arm64

Depends on: 3

- [ ] Accept aarch64 Linux in the host check, and report `linux_arm64` in
      the hardware report of an aarch64 machine.
- [ ] Set the native platform of the agent from the host architecture in
      `crates/ciproxy-agent/src/platform.rs`. Today `NATIVE` is
      `linux_x64` on every Linux build, so an arm64 agent would refuse its
      own runners and images with `platform_mismatch`.
- [ ] Read the processor name of an arm64 machine from a source that a
      64-bit arm64 kernel fills. Its `/proc/cpuinfo` has no `model name`
      line, so `crates/ciproxy-agent/src/hardware.rs` reports "Unknown
      processor" today.
- [ ] Pin the aarch64 Firecracker and jailer files of the pinned version
      with their SHA-256 values. Pick the pair of the host architecture.
- [ ] Give the VM configuration the boot arguments and the kernel file of
      each architecture.
- [ ] Pick the release target of the host architecture for agent updates and
      for the cache sidecar.
- [ ] Add unit tests for each architecture branch. Update
      `crates/ciproxy-agent/README.md`,
      `crates/ciproxy-agent/host-preparation.md`, and
      `crates/ciproxy-agent/updates-and-releases.md`.
- [ ] When decision 5 gives a host before the Pi, boot a Firecracker
      aarch64 VM with the VM configuration of the agent on that host. Keep
      the spike code and its evidence under
      `.context/linux-arm64-own-machines/`.

## Milestone 6: arm64 release builds and CI

Depends on: 5

- [ ] Build the `aarch64-unknown-linux-gnu` archives of the agent and the
      cache sidecar in `.github/workflows/agent-release.yml`.
- [ ] Package the arm64 archive in `.github/workflows/ci.yml`, as CI does
      for the macOS archive.
- [ ] Add a `rust-linux-arm64` gate phase: `cargo check` and `cargo clippy`
      of the Linux crates for `aarch64-unknown-linux-gnu`. Update
      `xtask/README.md`.
- [ ] Run the unit tests of the agent on a GitHub-hosted
      `ubuntu-24.04-arm` runner in `.github/workflows/ci.yml`.
- [ ] Add the third target to `deploy/release-server.md` and
      `crates/ciproxy-agent/updates-and-releases.md`.

## Milestone 7: The arm64 machine image

Depends on: 2, 5, and the arm64 build host of decision 5

- [ ] Pin the arm64 inputs in `image/pins.env`: the Ubuntu arm64 cloud
      image of the Ubuntu release of the x64 image, the `linux-arm64`
      runner, and the aarch64 guest kernel configuration of Firecracker.
- [ ] Build the patched Worker for `linux-arm64` with
      `image/build-runner.sh`.
- [ ] Build the guest, the cache sidecar, and the guest kernel for arm64.
- [ ] Add an aarch64 QEMU source for Packer, with UEFI firmware and KVM.
      Run the recipe that decision 1 chooses.
- [ ] Measure the image. Set the arm64 image budget and the disk minimum in
      `crates/ciproxy-protocol` and in the docs.
- [ ] Publish the arm64 image release beside the x64 release. Update
      `image/README.md`.

## Milestone 8: Dashboard and site

Tags: ui

Depends on: 1, 4

- [ ] Add Linux arm64 to the platform choice, the platform names, and the
      label form rules of `ts/app`, as the mockups show.
- [ ] Show arm64 machines on the machine pages, as the mockups show. Give
      an arm64 machine `ciproxy-arm64` as its default label. Today
      `defaultLabelName` in `ts/app/src/view/platform.ts` gives `ciproxy`
      to each machine that is not a Mac.
- [ ] Show the Boat switch only for the platforms that decision 3 sends to
      Boat. Today the dashboard tests for a macOS label instead.
- [ ] Add arm64 Linux and the Raspberry Pi to the install guide, the
      requirements, and the labels reference of `ts/site`.
- [ ] Run `npm --prefix ts run app:typecheck`, `npm --prefix ts run
      app:lint`, the app tests, and `npm --prefix ts run site:build`.
      Smoke-test the changed pages.

## Milestone 9: Live checks on an arm64 machine

Depends on: 5, 6, 7

- [ ] Let the smoke scripts of `crates/ciproxy-agent/scripts/` run on
      aarch64. Today `smoke-common.sh` and `smoke-host.sh` skip unless
      `uname -m` is `x86_64`.
- [ ] Install the agent from the release archive on an arm64 machine with
      KVM.
- [ ] Run the ext4 and KVM host smoke and the checks of
      `crates/ciproxy-agent/smoke-checks.md` there.
- [ ] Run a real GitHub job on `ciproxy-arm64`, with Docker and the cache.
- [ ] Prove the job isolation rules of `docs/protocol/security.md` on
      arm64.

## Milestone 10: Raspberry Pi checks

Depends on: 9, and the Pi from the user

- [ ] Install the agent on a Raspberry Pi 5 with 16 GB and an NVMe SSD, on
      64-bit Raspberry Pi OS and on Ubuntu.
- [ ] Check that Raspberry Pi OS takes the Debian path, and that the XFS
      and vsock modules and cgroup v2 work.
- [ ] Measure a job on the 16K-page and the 4K-page kernels. Propose the
      kernel rule as a decision, when one is needed.
- [ ] Check what install shows on 32-bit Raspberry Pi OS with its default
      64-bit kernel. There `uname -m` reports `aarch64`, so the upstream
      installer picks the arm64 agent, which a 32-bit system cannot start.
      Propose a clear message as a decision, when one is needed.
- [ ] Run a real GitHub job on the Pi. Name the Pi 5 as a tested machine in
      the docs.

## Milestone 11: Verification, commit, review, and pull request

Depends on: every other milestone

- [ ] Run the targeted tests and the complete gate named in `AGENTS.md`.
- [ ] Commit with Conventional Commits and push the branch.
- [ ] Review the complete local diff against `origin/main` with the
      `implementation-review` skill after the push. Report the findings,
      then apply the fix rule: fix the `fix now` findings, record
      follow-up items under `plans/follow-ups/` for the rest, re-review
      once, and report.
- [ ] Delete this plan when the `plans` skill says, under "Deleting the
      plan".
- [ ] Open the pull request. Its description gives the decisions and the
      design calls of this plan with their dates, a permalink to this plan,
      and the follow-up items that the change adds.

## Post-merge follow-up (non-blocking)

- [ ] Install a released agent on an arm64 machine against production, and
      run one real job on `ciproxy-arm64`.
