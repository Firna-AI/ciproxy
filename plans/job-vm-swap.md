# Compressed swap for Linux jobs

Status: Active. On branch `calummoore/swap-support-3dll`, no pull request
yet. The pull request that completes the work deletes this plan, as the
`plans` skill says.

A Linux job VM gets exactly the memory of its label and no swap. A
GitHub-hosted runner of 8 GB has about 7.75 GiB of memory and a swap file
of 3 to 4 GiB. So a job that passes on GitHub can run out of memory on the
`ciproxy` label, which configures 7.45 GiB and no swap. This change adds
compressed swap in memory, called zram, to each Linux job VM and each Boat
sandbox. It changes no disk rule, no slot rule, and no label size. It does
not cover macOS job VMs, because macOS manages its own swap. It does not
change what happens when a job uses more than its memory and its swap.
Decision 2 of the plan `job-step-cpu-and-memory.md`, on the branch
`calummoore/action-cpu-and-memory-usage`, covers that.

Contract owners:

- [Agent images](../docs/protocol/agent-images.md).
- [Labels](../docs/protocol/labels.md).
- The [image README](../image/README.md), the
  [image checks](../image/checks.md), and the
  [Boat build steps](../image/boat-build-steps.md).

## Terms

- **zram:** a Linux block device that keeps its data compressed in
  memory. Used as swap, it needs no disk.
- **zstd:** a compressor with a high ratio. The kernel option
  `CONFIG_ZRAM_BACKEND_ZSTD` lets zram use it.
- **Headroom:** the memory that a job can use beyond the memory of its
  label before the kernel stops a process.
- **MemTotal:** the memory that the guest kernel reports in
  `/proc/meminfo`. It is a little less than the configured memory.

## Design

- The machine build turns on `CONFIG_ZRAM`, `CONFIG_ZRAM_BACKEND_ZSTD`,
  and `CONFIG_ZRAM_DEF_COMP_ZSTD` as built-in options. The pinned
  Firecracker configuration has `CONFIG_ZRAM` off. `image/build-kernel.sh`
  sets the options with the kernel's `scripts/config` before
  `olddefconfig`. Decision 4 says what the build does when
  `olddefconfig` drops one of them.
- A new script `image/ciproxy-swap.sh` and a oneshot unit
  `image/ciproxy-swap.service` make the swap at boot:
  1. When a zram swap is already active, the script does nothing.
  2. It loads the `zram` module when the kernel has zram as a module.
  3. It selects zstd before it sets the size, because the kernel refuses
     a new compressor after the size is set. `zramctl --find --algorithm
     zstd --size` does both in this order. When the kernel has no zstd for
     zram, it keeps the kernel default and warns.
  4. It sizes the device at MemTotal, capped at 8 GiB.
  5. It runs `mkswap`, then `swapon` with priority 100.
  6. It sets `vm.page-cluster` to 0.
- When the kernel has no zram, or a step fails, the script warns and
  exits 0. The job then runs with no swap, as today.
- The machine image enables the unit before `ciproxy-guest.service`.
  `image/boat-start.sh` runs the script before it starts the Boat guest,
  because each Boat launch restores a snapshot.
- `vm.swappiness` stays at the kernel default of 60, as on GitHub.

### Design calls

All design calls are my calls, made on 23 November 2026.

- Size the device at MemTotal, capped at 8 GiB. Reason: zram holds the
  swapped pages compressed. At a 2:1 ratio, a device of about 7.3 GiB
  gives `ciproxy` about 3.6 GiB of headroom, close to GitHub's swap. Half
  the memory would give only about 1.8 GiB. The cap is the Fedora default
  and limits the memory that the zram tables use on large labels.
  Milestone 4 measures MemTotal in a job VM and corrects these figures.
- Use zstd. Reason: headroom is the goal, and zstd compresses better than
  lz4 or lzo. Rejected: lz4, which is faster but gives less headroom.
- Build zram into the kernel, not as a module. Reason: the machine image
  ships only `vmlinux` for the job VM kernel and no modules for it.
- Write an own script, not `systemd-zram-generator`. Reason: the script
  needs only `util-linux` and the kernel. A new package needs a new pin
  and differs between the two builds.
- Give the zram swap priority 100. Reason: when Boat adds its own disk
  swap, the kernel fills zram first.
- Keep `vm.swappiness` at 60. Reason: the kernel then swaps under the
  same pressure as on GitHub. A job that fits its memory can still swap
  some idle pages when the page cache grows, as on GitHub.
- Set `vm.page-cluster` to 0. Reason: zram has no seek cost, so reading
  ahead several pages only adds work.
- Set no `MemorySwapMax=` on the guest service. Reason: the runner and
  every step run in the control group of that service, so the setting
  would also keep the job out of swap.
- Warn and continue when zram is not available. Reason: a job without
  swap still runs as today, and the always-on rules prefer graceful
  handling.
- Check the swap at job start in `image/smoke-workflow.yml`, not in
  `image/check-runner-environment.py`. Reason: that check runs the guest
  in a chroot with no KVM, so it cannot see a zram device.
- The one-off parity test holds data that compresses like CI data: the
  files of `/usr` and the tool cache, read into memory. Reason: zram
  stores zero pages almost free and gains nothing on random data, so
  either would give a false result.

## Decisions

### 1. Linux job VMs have less memory headroom than GitHub

Answered on 23 November 2026: A. The options check of the same day added
E and F. The recommendation stays A.

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
- **Source:** Design.
- **Category:** Product bug.
- **Location:** `image/provision.sh:56`, `image/build-kernel.sh:18`, and
  `crates/ciproxy-agent/src/vm_config.rs:78`.
- **Background:** The image disables the Azure agent that makes swap on
  GitHub, and nothing else makes swap. The job VM kernel has swap and
  zswap, but not zram. Each job VM already gets a second drive file from
  the host, the Docker disk. On the smallest supported machine of 250 GB,
  one default job VM leaves only 1.6 GB of job disk free.
- **Problem:** A job that fits in the memory and the swap of a
  GitHub-hosted runner can run out of memory on a ciproxy label of the
  same size.
- **Impact if no action:** A customer who moves a memory-heavy job to
  ciproxy sees it fail and must pay for a larger label.
- **Root question:** How much memory headroom does a Linux job VM give a
  job, compared with a GitHub-hosted runner of the same size?
- **Options:**
  - **A.** Turn on zram in the kernel, and make a zram swap at boot in the
    job VM image and in the Boat image. Cost: compression uses the job's
    vCPUs, and data that does not compress gains nothing. Effort: Medium.
  - **B.** Give the guest the label memory plus 3 GiB. Cost: `ciproxy`
    then takes 2 slots, so each machine runs half as many default jobs,
    and Boat needs larger sizes. Effort: Medium.
  - **C.** Attach a sparse swap drive to each VM that takes only spare
    disk, as a Docker disk does. Cost: a machine with little spare disk
    gives little or no swap, and swap traffic slows the other jobs on the
    host disk. Effort: Large.
  - **D.** Give each VM a fixed 3 to 4 GiB swap that it reserves in the
    job disk. Cost: the 250 GB minimum machine then fits no default job,
    so the disk rule and the minimum change. Effort: Large.
  - **E.** Make a 3 GiB swap file at boot from the label's free job disk,
    as GitHub's `/swapfile` does. Cost: `ciproxy` then starts with about
    16.8 GB free, not 20 GB, so the free disk rule changes. Swap traffic
    shares the host disk. It works on any kernel. Effort: Medium.
  - **F.** State in `labels.md` that a Linux label gives at least the
    memory and swap of the GitHub runner of the same size. Add a step to
    the release smoke that holds that much data. Cost: a new release
    check, and the figures change when GitHub changes its runners. It can
    sit on top of A to E. Effort: Medium.
- **Left out:** Documentation only, because it fails the goal. Host memory
  lent through the balloon, virtio-mem, or host swap, because it lends
  memory beyond the slots and breaks the slot rule. A swap setting on each
  label, because alone it gives no headroom. zswap with a swap device
  that is never written, because A is better on every point: it needs a
  control group setting that systemd 255 cannot set.
- **Wider fix:** F. It finds a gap from any cause, such as a lost kernel
  option or a change of GitHub's swap. Decision 3 asks for it.
- **Previous examples:** The follow-up item `job-vm-swap.md` on the branch
  `calummoore/action-cpu-and-memory-usage`.
- **Recommendation:** A. It gives about the headroom of GitHub for typical
  data and changes no disk rule, slot rule, or machine minimum. If a Boat
  sandbox has neither zram nor swap, E on Boat is the fallback.
- **Action:** ask the user, because each option changes the memory or the
  disk that a job gets, and no requirement covers swap.

### 2. A label GB gives less memory than GitHub

Answered on 23 November 2026: A. The options check of the same day added
C. The recommendation stays A.

- **Severity:** Low. **Effort:** Small.
- **Source:** Design.
- **Category:** Docs or spec.
- **Location:** `docs/protocol/labels.md:72` and
  `crates/ciproxy-protocol/src/resources.rs:44`.
- **Background:** Label memory uses SI GB, so `ciproxy` configures
  7.45 GiB, and the guest probably sees about 7.3 GiB. The 8 GB runner of
  GitHub shows about 7.75 GiB.
- **Problem:** Before swap, a ciproxy job has about 0.3 to 0.45 GiB less
  memory than a GitHub job.
- **Impact if no action:** A job close to the limit can still fail. The
  swap of decision 1 covers this gap.
- **Root question:** Should the memory of a label match the actual memory
  of a GitHub-hosted runner?
- **Options:**
  - **A.** Keep SI GB, and let the swap of decision 1 cover the gap.
    Effort: Small.
  - **B.** Give memory in GiB and change the slot math to match. Cost:
    each machine fits fewer slots. Effort: Medium.
  - **C.** Give the VM the GiB figure of its label, and keep the label and
    the slots in SI GB. Cost: the extra memory comes from the host
    reserve, so a full machine keeps about 19.5% for the host, not 25%.
    Effort: Medium.
- **Left out:** Boat, because Boat sets the memory of its own sizes.
- **Wider fix:** F of decision 1, because it also finds this gap.
- **Previous examples:** None found.
- **Recommendation:** A. The gap is small next to the swap of decision 1,
  and B and C each change a slot rule or the host reserve.
- **Action:** ask the user, because B and C change a requirement in
  `labels.md` or `job-routing.md`.

### 3. No release check keeps memory parity with GitHub

Open. The options check of decision 1 found it on 23 November 2026.

- **Severity:** Low. **Effort:** Medium.
- **Source:** Review.
- **Category:** Test.
- **Location:** `image/smoke-workflow.yml` and `docs/protocol/labels.md`.
- **Background:** Milestone 4 runs the parity test once. The release smoke
  runs `image/smoke-workflow.yml` in a real job VM for each image release.
- **Problem:** Nothing checks the headroom after this change. A lost
  kernel option, a Boat kernel without zram, or a change of GitHub's
  runners can bring back the gap.
- **Impact if no action:** The gap comes back without a sign, until a
  customer's job fails.
- **Root question:** How does the team learn that a Linux label no longer
  gives the headroom of GitHub?
- **Options:**
  - **A.** No permanent check. The smoke step of Milestone 3 only checks
    that the swap is active. Effort: Small.
  - **B.** Add a step to the release smoke that holds the memory and swap
    of the GitHub runner in CI-like data, and state the parity in
    `labels.md`. Cost: a new release check that can fail, and figures to
    update when GitHub changes. Effort: Medium.
- **Left out:** A check in the repository gate, because the gate has no
  KVM and cannot start a job VM.
- **Wider fix:** B.
- **Previous examples:** None found.
- **Recommendation:** A. The active-swap step finds the likely failures,
  and a memory test in each release adds time and can fail on a change of
  data. Choose B for a hard promise.
- **Action:** ask the user, because B adds a release check that can fail.

### 4. The build fails when a zram option is lost

Open. The options check of decision 1 found it on 23 November 2026.

- **Severity:** Low. **Effort:** Small.
- **Source:** Review.
- **Category:** Process.
- **Location:** `image/build-kernel.sh:18`.
- **Background:** `olddefconfig` drops an option without an error when a
  dependency is missing. The image build runs in the release pipeline,
  not in the repository gate.
- **Problem:** A lost option gives a kernel without zram. The job VM then
  has no swap, and the script only warns.
- **Impact if no action:** A release can ship without swap, and only the
  release smoke shows it.
- **Root question:** When does the team learn that the job VM kernel lost
  zram?
- **Options:**
  - **A.** Fail the kernel build when an option is not `y` after
    `olddefconfig`. Cost: a new build error. Effort: Small.
  - **B.** Print a warning in the build log and continue. The smoke step
    of Milestone 3 then fails the release. Effort: Small.
- **Left out:** No check at all, because B costs the same and gives a
  sign.
- **Wider fix:** None, because the release smoke is already the wider
  check.
- **Previous examples:** None found.
- **Recommendation:** A. It stops a wrong kernel within minutes, not
  after a full image build and a smoke run.
- **Action:** ask the user, because A adds a new build error.

## Milestone 1: Documentation and protocol

Depends on: none

State the swap as a requirement of both builds and as part of what a label
gives a job.

- [ ] In `docs/protocol/agent-images.md`, under "One definition, two
      builds", state that a Linux job starts with a zram swap the size of
      its memory, capped at 8 GiB, in both builds. State the decision with
      its rejected options. State that a Boat kernel without zram gives no
      swap, and that the job still runs.
- [ ] In `docs/protocol/labels.md`, state under the size term that a Linux
      job also gets the compressed swap, and that the size itself does not
      change.
- [ ] Update `image/README.md` and `image/checks.md` with the kernel
      options, the swap unit, and the new checks.
- [ ] In `image/boat-build-steps.md` and the reason text of
      `image/boat_build_native.py:30`, say that the Boat build still
      removes the Azure swap, and that ciproxy adds its zram swap at
      launch.
- [ ] Update the site pages `ts/site/src/content/docs/docs/reference/labels.mdx`
      and `ts/site/src/content/docs/docs/how-ciproxy-works.md` with the
      swap of a Linux job.

## Milestone 2: zram in the job VM kernel

Depends on: 1

- [ ] In `image/build-kernel.sh`, turn on `CONFIG_ZRAM`,
      `CONFIG_ZRAM_BACKEND_ZSTD`, and `CONFIG_ZRAM_DEF_COMP_ZSTD` with
      `scripts/config` before `olddefconfig`.
- [ ] After `olddefconfig`, apply the answer of decision 4.
- [ ] Update the fixture of `image/check-kernel-metadata.py` for the new
      `scripts/config` call, and run the check.

## Milestone 3: The swap unit

Depends on: 2

- [ ] Add `image/ciproxy-swap.sh` and `image/ciproxy-swap.service`, as
      the design says.
- [ ] Install both in `image/provision.sh` in both modes. Enable the unit
      in the machine mode, ordered before `ciproxy-guest.service`.
- [ ] Run the script from `image/boat-start.sh` before the Boat guest
      starts.
- [ ] Add `image/check-swap.py`, which runs the script against a fake
      `/sys`, `/proc`, and fake tools. Cover the size cap, the order of
      the compressor and the size, the zstd fallback, the module load,
      the active swap, and the warning when zram is missing.
- [ ] Add a step to `image/smoke-workflow.yml` that checks that a zram
      swap of the expected size is active at job start.
- [ ] Apply the answer of decision 3.

## Milestone 4: Verification, commit, review, and pull request

Depends on: every other milestone

- [ ] Run the new image checks, the targeted tests, and the complete gate
      named in `AGENTS.md`.
- [ ] Build the machine image and start one job VM on a disposable KVM
      machine. Record MemTotal, and correct the figures of this plan and
      the docs. Check that `swapon --show` lists the zram device.
- [ ] Run a `ciproxy` job that holds 9.25 GiB of CI-like data, as the
      design calls say, and check that it passes.
- [ ] Start one Boat sandbox from a test snapshot. Record whether Boat
      already gives swap, and check that the zram swap is active. When
      the Boat kernel has no zram, ask the user about E of decision 1 on
      Boat.
- [ ] Commit with Conventional Commits and push the branch.
- [ ] Review the complete local diff against `origin/main` with the
      `implementation-review` skill after the push. Report the findings,
      then apply the fix rule: fix the `fix now` findings, record
      follow-up items under `plans/follow-ups/` for the rest, re-review
      once, and report.
- [ ] When `plans/follow-ups/job-vm-swap.md` is on `main`, delete it,
      because this change does its work.
- [ ] Delete this plan when the `plans` skill says, under "Deleting the
      plan".
- [ ] Open the pull request. Its description gives the decisions and the
      design calls of this plan with their dates, a permalink to this plan,
      and the follow-up items that the change adds.
