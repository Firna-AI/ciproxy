# Runner reports omit the hypervisor memory rounding

- **Severity:** Low. **Likelihood:** Common. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-22, from the branch
  `calummoore/macos-s-vm-memory-whole-mib`, review finding 2.
- **Location:** `crates/ciproxy-agent/src/macos/vm_spec.rs`,
  `VmSpec::settings`; `crates/ciproxy-agent/src/vm_config.rs`, the
  `mem_size_mib` of `MachineConfig`;
  `crates/ciproxy-agent/src/platform/macos/vms.rs`, `allocate`;
  `crates/ciproxy-protocol/src/runners.rs`, `RunnerReport::size`;
  `crates/ciproxy-control-plane/src/reconciliation.rs`, the call of
  `save_own_size`.
- **Reason:** The difference is below 1 MiB per VM, and one rule for
  both platforms reaches both VM backends and the meaning of a wire field.

- **Terms:**
  - **Label size:** the CPU count, memory, and disk of a label. Memory is
    in decimal GB, so 8 GB is 8,000,000,000 bytes (`GIGABYTE` in
    `crates/ciproxy-protocol/src/resources.rs`).
  - **Runner report:** `RunnerReport` in
    `crates/ciproxy-protocol/src/runners.rs`. The agent sends it for each
    VM. `requested_size` is the label size. `size` is documented as the
    "actual VM resources retained until removal".
  - **MiB:** 1,048,576 bytes. Firecracker and Virtualization.framework
    take memory in whole MiB.
- **Background:**
  - On Linux, `vm_config.rs` gives Firecracker
    `memory_bytes.div_ceil(1_048_576)` MiB. The VM gets up to 1 MiB more
    than the label.
  - On macOS, `VmSpec::settings` rounds the memory down to a whole MiB,
    because the VM must stay inside the label and its slot reserve. For
    an 8 GB label, the VM gets 7,999,586,304 bytes. For 16 GB, it gets
    15,258 MiB.
  - On both platforms, the agent reports the label bytes in both
    `requested_size` and `size`. The macOS `allocate` in
    `platform/macos/vms.rs` keeps the report as the control plane sent
    it.
  - The control plane saves `size` as the allocation of an own runner
    (`save_own_size` in `reconciliation.rs`). The scaler compares an
    actual size with a requested size in `covers` in
    `crates/ciproxy-scaler/src/size_policy.rs`. Today it does so only for
    Boat runners, not for runners on own machines.
- **Problem:** `size` says that the VM has the label memory, but the
  hypervisor gives it a little more on Linux and a little less on macOS.
  The two platforms round in opposite directions.
- **Impact if no action:** The reported allocation is wrong by less than
  1 MiB per VM. No current check fails on it. A later check of own runners
  that compares `size` with `requested_size` would need to know the rule.
- **Root question:** What memory does `RunnerReport::size` carry when the
  hypervisor takes whole MiB, and do both platforms use one rounding rule?
- **Options:**
  - **A.** Define `size` as the label allocation before the hypervisor
    rounding. Write this in the doc comment of `RunnerReport::size` and in
    `docs/protocol/agent.md`. Change no code. Effort: Small.
  - **B.** Report the exact hypervisor memory in `size` on both
    platforms. Use one rounding function per platform in the backend and
    in `allocate`. Keep `requested_size` as the label. Check that no
    consumer of `size` expects `size` to cover `requested_size`.
    Effort: Medium.
  - **C.** Round down on both platforms, so no VM uses more than its
    label, and report the exact memory as in B. Effort: Medium.
- **Left out:** A fix only in the report sender: the saved agent record
  would keep the label value, so a reconnect would send it again.
- **Wider fix:** C. One rule for both platforms removes the opposite
  rounding directions.
- **Previous examples:** Commit `fa5974dd` split the requested disk size
  from the allocated root size, so that the runner report keeps both.
- **Recommendation:** A. The difference is below 1 MiB, and a clear
  definition costs no code. Choose C when a consumer starts to compare
  the allocation of own runners with the label.
- **Action:** fix after merge, apply A, because the difference is below
  1 MiB and no current check depends on it.
