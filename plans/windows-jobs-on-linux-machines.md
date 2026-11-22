# Windows jobs on own Linux machines

Status: Active. On branch `calummoore/support-windows-jobs`, no pull request
yet. The pull request that completes the work deletes this plan, as the
`plans` skill says.

ciproxy runs the GitHub Actions jobs of a customer on the customer's own
machines first, and on Boat sandboxes when those machines are busy. Today a
Linux machine runs Linux jobs in Firecracker VMs, and a Mac runs macOS jobs
in macOS VMs. This plan adds Windows jobs. A job whose `runs-on` names a
Windows label runs in a fresh Windows Server VM on one of the customer's own
x64 Linux machines. A second hypervisor runs these VMs beside Firecracker,
because Firecracker cannot boot Windows. Each machine builds its own Windows
image from the customer's Windows Server licence, because no Microsoft
program lets ciproxy ship Windows.

This plan does not cover Windows machines as hosts, Linux jobs on Windows
machines, Windows on Arm, overflow of Windows jobs, or Docker disks for
Windows jobs.

Contract owners:

- The [architecture doc](../docs/architecture.md).
- The protocol docs [agent](../docs/protocol/agent.md),
  [agent install](../docs/protocol/agent-install.md),
  [agent images](../docs/protocol/agent-images.md),
  [labels](../docs/protocol/labels.md),
  [job routing](../docs/protocol/job-routing.md),
  [job routing capacity](../docs/protocol/job-routing-capacity.md),
  [overflow](../docs/protocol/overflow.md),
  [security](../docs/protocol/security.md),
  [cache](../docs/protocol/cache.md),
  [Docker disks](../docs/protocol/docker-disks.md), and
  [metering](../docs/protocol/metering.md).
- The READMEs of [ciproxy-agent](../crates/ciproxy-agent/README.md),
  [ciproxy-guest](../crates/ciproxy-guest/README.md),
  [ciproxy-cache-sidecar](../crates/ciproxy-cache-sidecar/README.md),
  [the image](../image/README.md), and
  [the runner patch](../image/runner-patch/README.md).

Replaced ways:

- Each machine has one platform and runs only the labels of that platform.
  This becomes: each machine has one host platform, and it reports the job
  platforms that it can run. A machine runs the labels of each job platform
  that it reports.
  - Consumers: the rule in `agent.md`, `labels.md`, and `job-routing.md`.
    `own_reservation.rs` in `ciproxy-store-pg`. `capacity.rs`, `warm.rs`,
    `snapshot.rs`, and `state.rs` in `ciproxy-scaler`. `snapshot.rs` and
    `own_start.rs` in `ciproxy-control-plane`. `label_views.rs`,
    `machines.rs`, and `machine_views.rs` in `ciproxy-api`.
    `view/platform.ts` and `api/parse-platform.ts` in `ts/app`.
  - Moved in Milestones 2, 5, and 11. Deleted in Milestone 5.
- The guest treats each build that is not macOS as Linux. This becomes: the
  guest names Linux, macOS, and Windows.
  - Consumers: `platform.rs` and `cli.rs` in `ciproxy-guest`.
  - Moved and deleted in Milestone 8.

## Terms

- **Windows job:** a job of a label with the platform `windows_x64`. It runs
  in a fresh Windows Server 2025 x64 VM. That is the OS of `windows-latest`.
- **Host platform:** the OS and the CPU of a machine, `linux_x64` or
  `macos_arm64`. It never changes.
- **Job platform:** the OS and the CPU that a job needs. A machine reports
  each job platform that it can run now.
- **Second hypervisor:** the VM program for Windows jobs on a Linux machine.
  It is Cloud Hypervisor, or QEMU if the spike rejects Cloud Hypervisor.
- **Windows recipe:** the open scripts and programs that build the Windows
  image on a machine. ciproxy publishes each recipe version. A recipe holds
  no Windows files.
- **Windows image:** the disk that a machine builds from a recipe version,
  the customer's ISO, and the customer's key.
- **viosock:** the Windows driver for virtio vsock, from the virtio-win
  project.

## Decisions

User decisions, 22 November 2026, from the brainstorm. Its notes are in
`.context/windows-jobs-on-linux-machines/brainstorm/`.

- Where Windows jobs run, item 1: A. They run on the customer's own Linux
  machines, with a second hypervisor beside Firecracker. Rejected: B,
  Windows machines with Hyper-V, because the agent then needs a full Windows
  port and customers need spare Windows machines. Rejected for now: C, both.
  Reason: A reaches every current customer with no new hardware, and it
  reuses the Linux isolation, storage, and vsock code.
- Where the Windows image comes from, item 2: A. Each machine builds its
  own image with ciproxy's open recipe. The customer gives a Windows Server
  ISO and a key at setup. Rejected: B, one customer build in customer
  storage, because the customer then runs storage. Rejected for now: C,
  ciproxy serves the image under a Microsoft agreement, because the legal
  result is unknown. Reason: A needs no Microsoft agreement and no customer
  storage, and the licence stays with the customer.

Design calls, my calls, 22 November 2026:

- A feasibility spike comes first. Reason: the Windows vsock driver, the
  boot time, and the build time are not proven.
- Job VMs use Cloud Hypervisor, with its UEFI firmware `CLOUDHV.fd` and its
  Hyper-V enlightenments. QEMU is the fallback. Reason: Cloud Hypervisor
  uses the vsock socket and the tap model of Firecracker, and it is Rust
  under Apache-2.0 and BSD-3-Clause.
- The image build installs Windows with QEMU from the packages of the Linux
  distribution. Reason: Cloud Hypervisor cannot run the Windows installer.
- The agent drives the image build with its own code, not with Packer.
  Reason: Packer has the Business Source License, and the macOS decision
  rejected tools with a limit from a third-party licence.
- The agent reaches the guest over vsock through viosock. The spike also
  tests a fallback: the agent connects to the guest over TCP inside the
  network namespace of the VM. Reason: a job VM must not reach the agent or
  the host, as [Job isolation](../docs/protocol/security.md#job-isolation)
  says.
- The customer gives the ISO and the key to the agent on the machine, with
  a new agent command. The key never leaves the machine. Reason: ciproxy
  then holds no Windows licence data.
- The recipe clears the key from the registry of the image with
  `slmgr /cpky`. The spike proves that job code cannot read the key. Reason:
  each job VM is a copy of the image, and job code runs as an administrator.
- The image build downloads virtio-win from the virtio-win project on the
  machine. ciproxy does not ship the signed drivers. Reason: their
  redistribution terms are not clear.
- Windows jobs get Docker with Windows containers in process isolation, as
  GitHub's Windows runners have. They get no Linux containers. They get no
  Docker disks at first, as Macs have none.
- Windows jobs use the machine cache from the first release. Reason: Linux
  machines already run it, and the cache sidecar has no OS-specific code.
- The root disk of a Windows VM has a fixed size, as the macOS disk has.
  Each VM gets a reflink copy of the image on the XFS area. Reason: a fixed
  disk needs no NTFS resize, and a reflink copy uses no space until the job
  writes.
- A machine keeps its ready Windows image while it builds the next recipe
  version. New VMs start from the newest ready image. Reason: the Linux
  images follow the same rule.
- Every workspace gets one default Windows label, `ciproxy-windows`, with
  2 vCPU and 8 GB.
- The smoke workflow gets a Windows mode. The test repositories, such as
  `Firna-AI/ciproxy-spike-cache`, get a copy.

## Open decisions

Items 3 and 4 come from the brainstorm. Item 5 is new. The options check of
the `plans` skill ran over all three on 22 November 2026. It added options
and changed each recommendation. Its report is in
`.context/windows-jobs-on-linux-machines/options-check-1.md`.

### 3. No overflow rule for Windows labels

- **Severity:** Medium. **Effort:** Small.
- **Source:** Design.
- **Category:** Docs or spec.
- **Location:** `docs/protocol/overflow.md:90-99`,
  `docs/protocol/labels.md:175-195`.
- **Background:**
  - Linux jobs overflow to Boat. Boat runs only Linux VMs.
  - macOS jobs never overflow, because cloud Macs bill for 24 hours at
    minimum.
  - The label setting "When nothing fits" asks the same question for a size
    that no machine fits.
  - AWS EC2 bills Windows per second with a 60-second minimum, and the price
    includes the licence. The foundation plan names AWS EC2 as the next
    backend.
  - Daytona sells Windows VM sandboxes by API, billed per second.
  - The scaler gives a free own slot to the waiting jobs in queue order,
    whatever their platform.
- **Problem:** No rule says where a Windows job goes when no own machine can
  take it.
- **Impact if no action:** The scaler and the label dialog have no
  behaviour for Windows labels.
- **Root question:** Where does a Windows job go when no own machine can
  take it now, because all are busy, none has a ready Windows image, or none
  fits its size?
- **Options:**
  - **A.** It waits for a free machine, as a macOS job does. Effort: Small.
  - **B.** It waits for now. When the AWS EC2 backend ships, that backend
    also starts Windows runners in the customer's AWS account. Cost: the AWS
    plan must add a Windows image. Effort: Small now, Large later.
  - **C.** One rule in the backend contract: a job overflows only to a
    backend that serves the platform of its label. Boat serves only Linux,
    so macOS and Windows jobs wait. The dialog and the API ask whether a
    backend serves the platform, not whether it is macOS. Each later backend
    names its platforms in its own plan. Cost: the macOS rule text moves into
    the general rule. Effort: Small.
  - **D.** C, and a free own slot goes first to a waiting job that no backend
    can take. Cost: in busy times, more Linux jobs go to Boat, and the
    customer pays for them. Effort: Medium.
  - **E.** A Windows sandbox backend in this plan, such as Daytona. Cost: a
    second backend with its own key, spend limit, and contract checks, and a
    recipe build inside the provider. Risk: Daytona gives Windows to Tier 3
    organizations only, on request, up to 4 vCPU and 50 GiB. Its licence
    terms for CI use are not checked. Effort: Large.
- **Left out:** Boat, because it runs only Linux VMs. GitHub-hosted Windows
  runners, because a ciproxy label reaches only its own runners, and the
  label router was rejected (`labels.md:22-27`). A job that fails at once,
  because "When nothing fits" rejected it (`labels.md:192-195`).
- **Wider fix:** C.
- **Previous examples:** The rule "macOS jobs never overflow", from
  3 November 2026.
- **Recommendation:** C. It behaves as A and B do in this plan, and it
  states the rule once, so that the dialog, the API, and the docs need no
  case for each platform. The first recommendation was B. The options check
  changed it.
- **Action:** ask the user, because no requirement covers overflow for a new
  label platform.

### 4. No software list for the Windows image

- **Severity:** Medium. **Effort:** Large.
- **Source:** Design.
- **Category:** Docs or spec.
- **Location:** `docs/protocol/agent-images.md:31-48`.
- **Background:**
  - The Linux and macOS images have all of the software of `ubuntu-latest`
    and `macos-latest`, so a job moves with a one-line change.
  - The user decision of 5 November 2026 keeps one image for each platform
    until launch, and rejects toolset layers for now.
  - The `windows-latest` software includes Visual Studio 2026. Build Tools
    can run on build machines when one person in the organization has a
    Visual Studio licence.
  - `container:` and `services:` jobs need Linux runners.
  - GitHub stops giving jobs to a runner that is more than 30 days behind a
    runner release. After a critical fix, it stops at once until the runner
    updates.
- **Problem:** Each machine builds the image, as item 2 chose. So the full
  software set costs each machine hours of build time and about 150 GB. With
  the runner inside the image, each runner release also rebuilds the image
  on each machine.
- **Impact if no action:** The recipe has no software list.
- **Root question:** Which software does a Windows job find in its VM, and
  what must each machine rebuild when part of it changes?
- **Options:**
  - **A.** A small base: Windows Server 2025, PowerShell 7, Git for Windows
    with Bash, the runner, and Docker with Windows containers. Jobs install
    the rest with setup actions. Cost: many jobs need more than a one-line
    change. Effort: Medium.
  - **B.** The small base, plus components that the customer selects once
    for the workspace from GitHub's scripts, so that each machine builds the
    same set. Examples: Build Tools, .NET SDKs, the Windows SDK, Node,
    Python, and Java. The customer confirms the licence of each paid
    component. Cost: the build time and the disk use grow with each
    component, and B reopens the one-image decision. Effort: Large.
  - **C.** All of the `windows-latest` software, as the other images have.
    Cost: about 4.5 hours of build time and 150 GB on each machine, and a
    Visual Studio licence. Effort: Large.
  - **D.** One set for every customer, chosen by ciproxy: GitHub's Windows
    software minus a published exclusion list. Build Tools goes in only when
    the customer confirms a Visual Studio licence. Cost: a job that needs an
    excluded part adds a setup step, and the list needs the spike's figures.
    Effort: Large.
  - **E.** D's set, and ciproxy's parts stay out of the slow build. The
    agent attaches the runner, its patch, the guest, and the cache sidecar
    to each VM on a small disk from each agent release. A runner release
    then needs no rebuild. This disk also works with A, B, or C. Cost: a
    start service in the image, and a check that a new runner works on an
    older image. Effort: Large.
- **Left out:** A set for each label, because it breaks the decision of
  5 November 2026. A shared tools disk, or tools that ciproxy ships, because
  they cut only the cheap part, and Build Tools needs a Visual Studio
  licence. One build shared between machines, because item 2 settled it.
  Job containers, because they run only on Linux.
- **Wider fix:** E.
- **Previous examples:** The Linux and macOS images, which have all of
  GitHub's software (`agent-images.md:31-34`, `agent-images.md:151-166`).
- **Recommendation:** E. It keeps the one-image decision of 5 November
  2026, which B reopens, and a runner release needs no rebuild of hours on
  each machine. The first recommendation was B. The options check changed
  it.
- **Action:** ask the user, because A, B, D, and E break the rule that an
  image has all of GitHub's software.

### 5. No Windows VM limit from the licence

- **Severity:** Medium. **Effort:** Medium.
- **Source:** Design.
- **Category:** Docs or spec.
- **Location:** `docs/protocol/job-routing.md:40-53`,
  `crates/ciproxy-protocol/src/platform.rs:26-31`.
- **Background:**
  - Windows Server Standard covers 2 VMs for each licensing of all the cores
    of a machine under volume terms. Under a hoster's SPLA terms, it covers
    1 VM for each licensing. Datacenter covers any number of VMs.
  - No Microsoft service tells ciproxy which licences a customer holds.
  - ciproxy starts as many VMs as the slots of a machine hold. A slot is
    2 vCPU and 8 GB.
  - A Mac has a fixed limit of 2 VMs, because of Apple's licence.
- **Problem:** A Linux machine with many slots can start more Windows VMs
  than a Standard licence covers. The image build VM also counts.
- **Impact if no action:** A customer with Standard can break the Microsoft
  licence and not know it.
- **Root question:** Who states how many Windows VMs a machine may run at
  once, and where does ciproxy keep that number?
- **Options:**
  - **A.** No limit. The docs and the agent command say that the customer
    must license every Windows VM that the slots allow. Effort: Small.
  - **B.** The customer sets a Windows VM limit for each machine, at setup
    and on the machine page. The default is 2, the Standard count. The
    scaler and the agent start no more. Effort: Medium.
  - **C.** The agent reads the edition of the built image. Datacenter gives
    no limit. Standard gives 2 for each licence that the customer declares.
    Effort: Medium.
  - **D.** The Windows agent command asks for the number, with a short
    guide for Standard, SPLA, and Datacenter, and with no default. The agent
    keeps the number with the key, refuses a start above it, and counts its
    own build VM. It reports a VM limit with each job platform. The same
    field carries the limit of 2 on a Mac, so the fixed table in
    `platform.rs` goes. The machine page shows the number and the command
    that changes it. Cost: a change of the number needs a shell on the
    machine. Effort: Medium.
- **Left out:** A pool for the workspace, because per-VM licences need
  Software Assurance and at least 8 core licences for each VM. A check with
  Microsoft, because no such service exists. Azure Arc pay-as-you-go,
  because each job VM would have to join Azure Arc with a credential that
  job code could read. A limit for each label, because the licence counts
  the VMs of one machine.
- **Wider fix:** D.
- **Previous examples:** The limit of 2 macOS VMs from Apple's licence
  (`agent.md:49-55`).
- **Recommendation:** D. It has no default that can break a Standard
  licence under SPLA terms, and it keeps the licence facts on the machine
  with the key. The first recommendation was B. The options check changed
  it.
- **Action:** ask the user, because a VM limit from a licence is new
  behaviour at a boundary that no requirement covers.

## Open questions

These questions need evidence first. Each one becomes a decision when the
evidence exists.

- How fresh job VMs activate Windows: through the customer's KMS host with
  the public KMS client key, or inside the 30-day grace period with no
  activation. The KMS client key also keeps the customer's key out of the
  job VMs. The spike shows the behaviour, and counsel confirms the licence
  terms.
- Whether the people or the devices whose jobs run in the VMs need Windows
  Server CALs. Counsel decides. The agent command and the docs then say it.
- Whether Windows labels get warm runners on Linux machines. The spike
  measures the cold start.
- Whether the agent can download Microsoft's evaluation ISO and convert it
  with the customer's key, so that the customer needs no ISO. Counsel
  decides.

## Milestone 1: Mockups

Tags: mockup

Depends on: none

The mockups show Windows labels and Windows jobs on Linux machines in each
product area of this plan.

- [ ] Label dialog: the platform choice gets Windows. A Windows label has no
      Boat switch, and it shows the busy rule of item 3.
- [ ] Label list and default labels: the platform of each label, and
      `ciproxy-windows`.
- [ ] Machine page: the job platforms of a Linux machine, the Windows VM
      limit of item 5, and each state of its Windows image. The states are
      "no ISO and key", "building" with progress, "ready" with its recipe
      version, and "failed" with its cause.
- [ ] Machines list, Setup, and Add machine: a Linux machine that turns on
      Windows jobs, with the agent command and the licence confirmation.
- [ ] Jobs: a Windows job in the job list and on the job page.
- [ ] Site mockups: the guide "Run Windows jobs", the labels reference, and
      the plan text.
- [ ] The component choice of item 4, when the answer to item 4 needs one.
- [ ] Run `npm --prefix ts run mockups:check`. Check the changed screens at
      1440, 720, and 390 px.
- [ ] Update the design and the TODOs of this plan for each problem that the
      mockups show.

## Milestone 2: Documentation and protocol

Depends on: 1

The docs state the requirements and the decisions for Windows jobs.

- [ ] `architecture.md`: Windows job VMs on Linux machines over the second
      hypervisor, and the Windows image that each machine builds.
- [ ] `agent.md`: the host platform and the job platforms of a machine, the
      Windows job VMs, the Windows image build, and the mismatch rule.
- [ ] `agent-install.md`: the command that turns on Windows jobs, what it
      needs, and what it does.
- [ ] `agent-images.md`: the Windows image, its recipe, its build on each
      machine, and its software from item 4.
- [ ] `labels.md`: the Windows platform, the default label, and the busy
      rule of item 3.
- [ ] `job-routing.md` and `job-routing-capacity.md`: the job platform
      match, the Windows VM limit of item 5, and the Windows disk budget.
- [ ] `overflow.md`: the rule of item 3.
- [ ] `security.md`: the isolation of a Windows job VM, the IPv6 block, and
      the licence data that stays on the machine.
- [ ] `cache.md` and `docker-disks.md`: the machine cache for Windows jobs,
      and no Docker disks for them.
- [ ] `metering.md`: Windows jobs in the savings report, at the Windows list
      prices of each alternative.
- [ ] Run `cargo xtask check --include markdown`.

## Milestone 3: Feasibility spike

Depends on: none

Spike code stays under `.context/windows-jobs-on-linux-machines/spikes/`,
not on the branch. Its `README.md` gives the steps. The spike needs an x64
Linux machine with KVM and a Windows Server 2025 ISO. Evaluation media is
acceptable here, because the spike is a test.

- [ ] Install Windows Server 2025 from the ISO into a base disk with QEMU,
      an answer file, and no screen. Install viostor, NetKvm, and viosock.
      Record the time.
- [ ] Boot a reflink copy of the base disk under Cloud Hypervisor, in a
      network namespace with a tap. Pass: a guest service runs in under
      60 s.
- [ ] Connect from the host to a guest listener through the vsock socket
      with `CONNECT 5252` and viosock. Pass: 1,000 connections and JSON
      exchanges with no hang. When it fails, prove the TCP fallback.
- [ ] Run one real GitHub job with a just-in-time `win-x64` runner in a
      fresh copy. Pass: the job passes, and the VM stops when the runner
      exits.
- [ ] Run a Windows container with process isolation in the guest.
- [ ] Prove that the nftables rules leave the guest the internet and the
      machine cache port only. Prove that IPv6 is blocked.
- [ ] Record the activation state of a fresh copy with and without sysprep,
      and the boot time that sysprep adds.
- [ ] Prove that job code in a fresh copy cannot read the customer's key
      after `slmgr /cpky`.
- [ ] Build the small base of item 4. Record its build time and its disk.
- [ ] Save and restore a guest under Cloud Hypervisor. Record the time and
      each failure.
- [ ] When Cloud Hypervisor fails a pass mark, run the same steps with QEMU.
      Raise the choice as a decision.
- [ ] Record the results. Raise the activation question and the warm-runner
      question as decisions, with the measured times.

## Milestone 4: Protocol types

Depends on: 2

`ciproxy-protocol` carries the Windows job platform, the job platforms of a
machine, and the Windows image build.

- [ ] Add `windows_x64` to `Platform` for labels, runner starts, and recipe
      downloads. A machine never reports it as its host platform.
- [ ] Add the job platforms of a machine and the state of its Windows image
      to the hardware report.
- [ ] Add the Windows recipe download, with its version, its SHA-256, its
      size, and its signed address.
- [ ] Add the Windows VM limit of item 5 to the messages that need it.
- [ ] Update the snapshots and the wire-value tests.

## Milestone 5: Control plane

Depends on: 4

The control plane sends Windows jobs to Linux machines with a ready Windows
image.

- [ ] Store: a migration lets labels hold `windows_x64`. It stores the job
      platforms and the Windows image state of each machine.
- [ ] Scaler: a Windows job goes only to a machine that reports
      `windows_x64`. Apply the Windows VM limit and the busy rule of item 3.
      Delete the rule that the label platform equals the machine platform.
- [ ] Boat: keep the guard that accepts only Linux labels.
- [ ] Recipes: read the Windows recipe from the image manifest. Send it to
      each machine that turned on Windows jobs, as image delivery sends the
      Linux image.
- [ ] API: `LabelInput.platform` takes `windows_x64`. `MachineView` gives
      the job platforms and the Windows image state. Make the default label
      `ciproxy-windows`.
- [ ] Savings: add the Windows list prices of GitHub and Blacksmith. When an
      alternative sells no Windows runner, raise a decision.
- [ ] Regenerate the API types with `npm --prefix ts run api:generate`.

## Milestone 6: Agent, Windows job VMs

Depends on: 3, 4

The Linux agent starts Windows job VMs under the second hypervisor, beside
its Firecracker VMs.

- [ ] Pin the second hypervisor and its firmware by SHA-256, as
      `host_release.rs` pins Firecracker.
- [ ] Add a Windows VM backend behind `Vms`. Each VM gets a systemd unit,
      the network namespace and the tap of the Linux VMs, a reflink root
      disk, and no Docker drive. The platform of the runner start picks the
      backend.
- [ ] Reuse `GuestChannels` with the vsock socket, or with the TCP fallback
      that the spike chose.
- [ ] Reattach running Windows VMs after an agent restart or update.
- [ ] Apply the nftables rules of the Linux VMs. Check the IPv6 block.
- [ ] Refuse a Windows runner start when no Windows image is ready.

## Milestone 7: Agent, Windows image build

Depends on: 3, 4

The agent builds and keeps the Windows image of its machine.

- [ ] Add the agent command that takes the ISO path and the key, and asks
      the customer to confirm the licence. Keep the key on the machine,
      readable by root only.
- [ ] Install QEMU and the build tools from the packages of the
      distribution, as install adds its other packages.
- [ ] Build from the newest recipe when the machine has free slots. Count
      the build against the slots, the disk budget, and the Windows VM limit
      of item 5, because the licence counts the build VM.
- [ ] Download virtio-win on the machine. Check its SHA-256 against the
      recipe.
- [ ] Keep the old ready image until the new image is ready. Remove each
      image that no VM holds.
- [ ] Report the image state, the build progress, and each failure cause.
- [ ] Make uninstall remove the Windows images, the ISO copy, and the key.

## Milestone 8: Guest, cache sidecar, and runner patch on Windows

Depends on: 3

The programs inside the VM build and run on `x86_64-pc-windows-msvc`.

- [ ] `ciproxy-guest`: add a Windows platform. The guest runs as a Windows
      service and listens on vsock port 5252 through viosock, or on the TCP
      fallback. It starts the runner as the runner account, and it powers
      off the VM when the runner exits.
- [ ] Give the guest a Windows assignment socket that the runner patch can
      reach.
- [ ] Start the cache proxy on `127.0.0.1:8230` and on the address that
      Windows containers reach.
- [ ] Build `ciproxy-cache-sidecar` for Windows, and run its tests there.
- [ ] Build the patched `Runner.Worker.dll` for `win-x64` on a Windows
      computer, as `image/build-runner.sh` builds the other runtimes.
- [ ] Add a CI job on a GitHub-hosted Windows runner. It tests the guest and
      the cache sidecar, and it builds the runner patch.

## Milestone 9: Windows recipe

Depends on: 3, 8

`image/windows/` holds the open recipe that each machine runs.

- [ ] Write the answer file, the driver install, and the network setup of
      the guest.
- [ ] Install the software of item 4. Use the actions/runner-images Windows
      scripts where they run outside Azure.
- [ ] Install the guest service, the cache sidecar, the runner, and its
      patch. With option E of item 4, install only the start service that
      runs them from the disk that the agent attaches.
- [ ] Run sysprep as the spike chose, then check the finished image.
- [ ] Package each recipe version with its SHA-256. Publish it as
      `image/package.sh` and `image/publish.sh` publish the Linux image.

## Milestone 10: Release and gate

Depends on: 8, 9

The release builds the Windows programs, and the gate checks them.

- [ ] `agent-release.yml` builds the Windows guest programs for the recipe.
- [ ] A `rust-windows` gate phase type-checks the Windows crates for
      `x86_64-pc-windows-msvc`, as `rust-macos` does for macOS.
- [ ] `updates-and-releases.md` and the scripts README list the Windows
      parts.

## Milestone 11: Dashboard and site

Tags: ui

Depends on: 1, 5

The app and the site show Windows labels and Windows jobs, as the mockups
show.

- [ ] App: the label dialog, the label list, the machine page, Setup, and
      Jobs.
- [ ] Label form: turn Boat off for each platform that no backend serves.
      Today `view/settings-label-form.ts` turns it off for macOS only, so
      the API would refuse a new Windows label.
- [ ] Site: the guide "Run Windows jobs", the labels reference, the agent
      install reference, and the plan text.
- [ ] Run the app and site checks named in `AGENTS.md`.

## Milestone 12: Smoke on a real machine

Depends on: 6, 7, 9, 10

A real Windows job runs end to end before the merge.

- [ ] Add a Windows mode to
      `crates/ciproxy-agent/scripts/github-smoke-workflow.yml`.
- [ ] On a test machine with the staging control plane, turn on Windows
      jobs, build the image, and run the Windows smoke job.
- [ ] Copy the workflow to the test repositories.

## Milestone 13: Verification, commit, review, and pull request

Depends on: every other milestone

The complete gate, the review, and the pull request.

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

- [ ] Run the Windows smoke job against production.
- [ ] Add Docker disks for Windows jobs.
- [ ] Add Windows overflow on AWS EC2, when item 3 chooses it.
