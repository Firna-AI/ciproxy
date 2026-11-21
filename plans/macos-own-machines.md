# macOS jobs on own Macs

Status: Active. On branch `calummoore/support-mac-os-kq1u`, no pull request
yet. The pull request that completes the work deletes this plan.

ciproxy runs the GitHub Actions jobs of a customer on the customer's own
machines first, and on Boat sandboxes when those machines are busy. Today
every own machine is an x64 Linux computer, and every job runs Linux. This
plan adds macOS jobs: a job whose `runs-on` names a macOS label runs in a
fresh macOS VM on one of the customer's own Apple Silicon Macs.

Contract owners:

- The architecture doc, `docs/architecture.md`.
- The protocol docs under `docs/protocol/`, mainly `agent.md`,
  `agent-install.md`, `agent-images.md`, `labels.md`, `job-routing.md`,
  `security.md`, and `billing.md`.

## Terms

- **Mac:** an own machine with Apple Silicon that runs macOS and the agent.
  The product calls it a machine, as it does a Linux computer.
- **macOS label:** a label whose jobs run in a macOS VM on a Mac.
- **Virtualization.framework:** Apple's API that runs VMs on a Mac. It runs
  macOS guests only on Apple Silicon.
- **macOS image:** the job image for macOS VMs. It holds macOS, Xcode, the
  software of the GitHub runner image that `macos-latest` names, the
  runner, and the guest.
- **APFS clone:** a copy-on-write copy of a file on the Apple file system.
  It takes almost no time and no space at first.

## Decisions

User decisions, 21 November 2026:

- Scope: macOS jobs on the customer's own Macs. Rejected: Linux arm64 jobs
  on Macs, because a workflow that needs Xcode still could not run.
  Rejected for now: both.
- Order: a feasibility spike on a real Mac first. Then the build runs in
  parallel workspaces, because production is almost live.
- VM layer: our own Rust code over Virtualization.framework, through the
  `objc2-virtualization` crate. Rejected: Tart and Lume as external tools.
  Reason: one agent binary, the Apache-2.0 licence of the agent, and no
  limit from a third-party licence.
- Image: ciproxy builds and serves a ready macOS image with Xcode, as it
  does for the Linux image. The user has approval for this distribution.
  Rejected: an image that each Mac builds itself, and a customer image.
- Billing: a Mac counts as one own machine, on every tier, at the price of
  any own machine.

Earlier decision, 3 November 2026, in the MVP foundation plan: macOS runs
on own Macs only. Reason: cloud Macs rented by API bill 24 hours at minimum
under Apple's licence, so per-minute macOS overflow is not possible. So a
job of a macOS label never goes to Boat. It waits for a free Mac.

## Constraints

- Apple's licence allows at most 2 macOS VMs at a time on one Mac. So a Mac
  has at most 2 slots for macOS jobs, whatever its size. The spike checks
  the exact limit and its error.
- The agent signs with the `com.apple.security.virtualization`
  entitlement. A release build also needs an Apple Developer ID and
  notarization.
- The agent crate keeps its Apache-2.0 licence and depends on no workspace
  crate without one.

## Milestone 1: Feasibility spike

Spike code stays under `.context/macos-own-machines/spikes/`, not on the
branch. Its `README.md` gives the steps. Each item names its evidence.
The spike needs an Apple Silicon Mac with macOS 14 or later.

- [x] Install macOS from Apple's restore image into a base VM with our own
      Rust code, then finish the Setup Assistant in a window.
      Passed on a Mac mini M4 with a macOS 26.6.2 guest: install in 169 s.
      The Setup Assistant ran over the private `_VZVNCServer`, with no
      screen on the Mac.
- [x] Clone the base VM with APFS clones and cold-start the clone to an SSH
      login. Pass: a clone in under 1 s, SSH in under 60 s.
      Passed: a clone in 1 ms, SSH login in 11.4 s.
- [ ] Run one real GitHub job on a just-in-time runner in a fresh clone.
- [x] Find the limit of concurrent macOS VMs and its error text.
      Passed: the limit is 2, and the third start fails at once.
- [x] Save a VM state, restore it, and measure the time to SSH. Check
      whether two restores of one state run at once.
      Passed: SSH 3.6 s after a restore. One saved state serves one VM at
      a time, so a Mac keeps one saved state for each slot.
- [x] Prove that pf rules leave a guest the internet only, as
      [Job isolation](../docs/protocol/security.md#job-isolation) says.
      Passed after a rule that blocks all IPv6 from the guests. Without
      it, a guest reached the host over IPv6 link-local.

Evidence: `.context/macos-own-machines/spikes/vz/RESULT.md`.

## Milestone 2: Build, in parallel workspaces

Depends on: 1. Each item becomes its own sub-workspace and pull request.

- [ ] Protocol and docs: add the platform to each label and each machine.
      Route a macOS label only to Macs, with no Boat overflow. Cap the
      macOS slots of a Mac at the licence limit. Update the contract docs.
- [ ] Agent platform layer: put the Linux-only parts of `ciproxy-agent`
      behind one platform interface, with no change in Linux behaviour.
      These parts are systemd, Firecracker, ext4 and `e2fsprogs`, the
      network rules, and the `/proc` and `/sys` reads.
- [ ] Agent on macOS: the Virtualization.framework VM layer, APFS clones,
      launchd, pf isolation, the hardware report, and the macOS paths of
      install, update, and uninstall. Depends on the platform layer.
- [ ] macOS image: an automatic build from Apple's restore image and the
      macOS scripts of the GitHub runner images, with Xcode, the runner,
      and the guest. Automate the Setup Assistant. Serve the image from
      the download site.
- [ ] Guest, cache sidecar, and runner patch on macOS, with the cache
      handoff of [The cache handoff](../docs/protocol/cache-handoff.md).
- [ ] Control plane: the platform in the store, the API, the scaler, and
      the gateway.
- [ ] Dashboard and site: the platform in the label dialog and on the
      machine page, the Mac install guide, and the tier text "own Linux
      machines".
- [ ] Release: a signed and notarized macOS arm64 agent, and the macOS path
      of the install script.

## Open questions

- Which Apple Developer ID signs the agent, and who holds it.
- Where the macOS image lives, because it is far larger than the Linux
  image.
- Whether a warm runner on a Mac uses a restored state, after the spike
  measures it.
