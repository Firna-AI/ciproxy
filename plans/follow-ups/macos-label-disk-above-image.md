# A macOS label cannot ask for more disk than its image

- **Severity:** Low. **Effort:** Large. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-23, from the branch `calummoore/macos-s-disk-real-use`,
  point 4 of its task.
- **Location:** `crates/ciproxy-agent/src/root_sizing.rs`, `RootRule::root_bytes`;
  `crates/ciproxy-agent/src/platform/macos/vm_files.rs`, `clone_with`;
  `crates/ciproxy-guest`; `crates/ciproxy-api/src/handlers/label_views.rs`;
  `crates/ciproxy-api/src/handlers/machine_views.rs`.
- **Reason:** A larger disk needs the clone to grow and the guest to resize
  its APFS container at boot, which is not a small change.

- **Terms:**
  - **Free job disk:** the disk that a label gives each job, in decimal GB.
  - **Image free bytes:** `free_job_bytes` of a macOS image release: the
    free space inside its fixed `disk.img` at the end of the build. The
    standard release has a 100 GB disk with 21.8 GB free.
  - **APFS clone:** a macOS job VM disk is a `clonefile` copy of the image
    `disk.img`. It has the same logical size as the image disk.
- **Background:**
  - `RootRule::root_bytes` gives no root when a macOS label asks for more
    free disk than the selected image has. So the scaler places no job of
    that label on that Mac, and the agent refuses such a start with the
    warning "the label asks for more free disk than the image gives".
  - The label dialog shows no limit. For such a label, the Mac fit in
    `label_views.rs` shows 0 jobs and no disk size, and the Mac page gives
    no advice to add disk. It names no reason.
  - `docs/protocol/labels.md`, "The disk of a macOS label", says that the
    dialog is planned to show the limit and refuse a larger value.
  - `docs/protocol/agent-images.md` decides a fixed image disk: a guest
    that resizes its container at each boot was rejected, also because a
    saved VM state needs the same disk.
- **Problem:** A customer cannot give a macOS job more disk than the image
  has, and the dialog does not say why such a label never runs on a Mac.
- **Impact if no action:** A macOS label above about 21.8 GB waits with no
  clear reason in the dashboard.
- **Root question:** How does a macOS label get a clear answer when it asks
  for more disk than the image has?
- **Options:**
  - **A.** Show the image free bytes as the limit in the label dialog, and
    refuse a larger value there. Effort: Medium.
  - **B.** Grow the clone's `disk.img` to the label disk before the start,
    and let `ciproxy-guest` resize the APFS container at boot. A saved
    state then serves only one disk size. Effort: Large.
  - **C.** Publish image releases with larger disks, and let a label pick
    one. Effort: Large.
- **Left out:** A refusal at the agent only: the control plane would try
  the start again on each pass.
- **Wider fix:** A. One limit in the dialog covers every macOS image.
- **Previous examples:** None found.
- **Recommendation:** A. It gives the clear reason with no guest change.
- **Action:** fix after merge, apply A.
