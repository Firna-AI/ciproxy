# Network cleanup drops durable ownership

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-04, from
  `conductor/ciproxy-mvp-milestone-9-agent`, review item 9.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-agent/src/network.rs`,
  `crates/ciproxy-agent/src/vms.rs`,
  `crates/ciproxy-agent/src/observations.rs`,
  `crates/ciproxy-agent/src/launches.rs`, and
  `crates/ciproxy-agent/src/controller.rs`.
- **Reason:** This Rare defect needs a host cleanup failure, and a later
  start retries cleanup after a transient failure.

- **Terms:**
  - **Network index:** A number that selects a VM's namespace, virtual
    interfaces, host addresses, and firewall table.
  - **VmRecord:** The durable ownership record for one agent VM.
  - **Reconciliation:** The agent checks owned VMs and retries pending
    work.
  - **Slot:** Two hardware threads and 8 GB of job memory.
- **Background:** Each fresh VM owns network resources named from its
  index, such as `/run/netns/cpxN`, a virtual link, and an nftables table.
  VM retirement stops the VM and removes those resources before deleting
  its VmRecord. Allocation takes the lowest index that no VmRecord holds.
- **Problem:** `LinuxNetwork::remove` logs every failed `ip` or `nft`
  deletion, then returns success. It cannot distinguish an absent resource
  from a failed deletion. Retirement and failed-start cleanup consequently
  delete ownership even when resources remain on the host.
- **Impact if no action:** The next VM can reuse the index and fail at
  `ip netns add`. A lasting host failure can block every retry on that
  index. Remove and uninstall can leave resources until the host restarts.
- **Evidence:** `LinuxNetwork::remove` returns `Ok(())` after its deletion
  loop. `LinuxVms::stop` therefore succeeds, and `Controller::retire` removes
  the record. The failed-start path in `launches.rs` also releases the
  record after that success. The command boundary in `process.rs` gives
  these failures the same `HostCommand` error.
- **Requirement:** `docs/protocol/agent.md` requires fresh VMs for jobs and
  preserves running jobs during removal. Network cleanup must keep new
  starts available without keeping a completed VM in job or warm reports.
- **Root question:** How can the agent keep leftover network resources
  from blocking new VMs and remove them in the end?
- **Options:**
  - **A.** Verify absence after a deletion failure. Return an error when a
    resource remains or verification fails. Keep ownership for retry.
    Finished VMs then keep their report and slots until cleanup succeeds.
    Effort: Small.
  - **B.** Give deletion typed outcomes for removed, already absent, and
    failed resources. Use them during retirement and recovery. This still
    needs host inspection to classify absence. Effort: Medium.
  - **C.** Persist pending cleanup separately from completed VMs. Reserve
    its indices until cleanup succeeds. This changes durable state and
    still needs reliable host inspection. Effort: Medium.
  - **D.** Give network resources one owner. Before allocation, inspect
    host resources and skip every occupied index. During reconciliation,
    retry cleanup of resources that no VM owns. Include failed-start
    leftovers. This adds host-list commands and output parsing.
    Effort: Medium.
  - **E.** Tie network lifetime to each VM's systemd unit. Let the kernel
    remove its namespace and link pair. Keep host rules in one agent
    table. This replaces network setup and needs real KVM validation.
    Effort: Large.
- **Left out:** Ignoring all errors cannot prove cleanup. Returning every
  command error rejects normal removal of already-absent resources.
  Matching error text depends on host locale and violates the Rust error
  contract. Rotating allocation only delays collisions. Unique VM names
  alone leave the index-derived host addresses in conflict.
- **Wider fix:** D. The common allocator never selects an index whose
  network resources still exist.
- **Previous examples:** The image owner in
  `crates/ciproxy-agent/src/image_registry.rs` reclaims unowned files and
  retries cleanup. The network resources need the same ownership pattern.
- **Related:** Runtime removal is already terminal in
  `crates/ciproxy-agent/src/mode.rs`. The new owner must preserve that
  behavior when reconciliation or cleanup waits.
- **Recommendation:** D. Host resource ownership protects both allocation
  and eventual cleanup without keeping completed jobs in reports.
- **Action:** fix after merge, apply D, because a host cleanup failure is
  Rare and the change adds no durable format that gets harder to replace.
- **Done when:** Failed deletion leaves its index unavailable to a new
  VM. Reconciliation removes leftovers after host recovery. Remove and
  uninstall finish only when no agent-owned resource remains. After the VM
  and its disks are removed, network cleanup failure does not delay
  `RunnerExited` or retain its slots. Failed-start cleanup receives the
  same protection. Tests through
  `Commands` fakes cover failed deletion, skipped allocation, and later
  cleanup. Reports remain independent of cleanup waits. The crate README
  states ownership of the `cpxN` namespace. The change adds no host tool or
  kernel requirement without separate approval.
