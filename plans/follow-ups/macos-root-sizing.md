# macOS VMs use the Linux root disk sizing

- **Severity:** Low. **Likelihood:** Common. **Effort:** Medium.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-22, from the branch
  `calummoore/macos-s-vm-memory-whole-mib`, round 2 of its review.
- **Location:** `crates/ciproxy-agent/src/launches.rs`, the
  `RootSizing::for_image(&metadata).vm_size(runner.size)` call in
  `launch`; `crates/ciproxy-agent/src/root_sizing.rs`, `root_bytes`.
- **Reason:** The fix decides how a macOS VM disk is counted, which is a
  question apart from the memory fix of that branch.

- **Terms:**
  - **Root sizing:** `RootSizing` in `root_sizing.rs`. It turns the free
    job disk of a label into the full root disk of a Linux VM: the image
    root, the free bytes, a 1% root reserve, 2% new ext4 metadata, and
    the ext4 reserve pool, rounded up to 4096 bytes.
  - **APFS clone:** a macOS VM disk is a `clonefile` copy of the image
    `disk.img`. It keeps the fixed size of the image disk. The label disk
    size never reaches Virtualization.framework.
- **Background:** `launch` in `launches.rs` is shared by both platforms.
  It sizes every VM with `RootSizing`, checks the result against the free
  job disk of the machine, and reports it as `RunnerReport::size`.
- **Problem:** On macOS, the check and the report use an ext4 root size
  that the VM never gets. The real disk use of a macOS VM is the APFS
  clone, which starts near zero and grows to at most the image disk.
- **Impact if no action:** The macOS capacity check counts a disk size
  that does not match the VM. The report shows a root size that the VM
  does not have.
- **Root question:** How does the agent count and report the disk of a
  macOS VM?
- **Options:**
  - **A.** Keep the shared sizing, and document that macOS counts the
    label disk as a reserve. Effort: Small.
  - **B.** Give the platform backend a sizing step, so macOS counts the
    image disk size. Effort: Medium.
  - **C.** Count the APFS clone growth from the area volume use, not from
    a size per VM. Effort: Large.
- **Left out:** A change of the label disk sizes: the label stays one
  rule for both platforms.
- **Wider fix:** B. Each backend owns the sizing of its own disks.
- **Previous examples:** Commit `fa5974dd` split the requested disk from
  the allocated root size for Linux.
- **Recommendation:** B. It makes the check and the report match the VM.
- **Action:** fix after merge, apply B, because it is a separate design
  question of the macOS backend.
