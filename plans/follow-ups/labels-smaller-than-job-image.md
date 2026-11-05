# Labels smaller than the job image

- **Severity:** Medium. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-05, from `conductor/ciproxy-mvp-73a-disk-sizes`.
- **Location:** `crates/ciproxy-agent/src/vm_root.rs`,
  `crates/ciproxy-scaler/src/capacity.rs`,
  `crates/ciproxy-scaler/src/size_policy.rs`, `docs/protocol/labels.md`,
  `docs/protocol/job-routing.md`, and `docs/protocol/overflow.md`.
- **Reason:** The user asked to keep the capacity error and record the
  smaller-label policy for a later decision.

- **Source:** Implementation.
- **Category:** Docs or spec.
- **Terms:**
  - **Label:** The name in a workflow's `runs-on` line that selects a VM
    size, including its disk.
  - **Image root disk:** The prepared ext4 disk that the agent copies for
    each job VM.
- **Background:**
  - On 5 November 2026, the user chose the full software of GitHub's
    `ubuntu-latest` image and an 80 GB default job disk.
  - Each VM gets one root disk of its label size. Docker data shares that
    disk and its free space.
  - The agent makes a sparse reflink copy of the image root disk. It
    extends the copy and grows ext4 when the label disk is bigger.
  - The agent keeps its existing capacity error when the label disk is
    smaller than the image root disk. It never shrinks the full image.
  - The minimum is the logical length of the image root disk. The approved
    full-image release uses 80 GB.
  - The control plane does not receive that minimum today. Each option
    needs the root size of each image release, or one protocol constant.
  - The explicit largest-Boat fallback in `size_policy.rs` can select an
    actual disk below the image floor, even when the label is larger.
    Include that fallback when applying the chosen image-fit policy.
- **Problem:** A custom label can request less disk than the image needs.
  The current label rules do not choose between a larger actual VM disk
  and refusal of that label in settings.
- **Impact if no action:** The scaler can place an undersized label on an
  own machine whose agent refuses it. A label of 51 GB or less can also
  select a Boat size that cannot hold the full image.
- **Root question:** What happens to jobs whose label disk cannot hold the
  job image, on own machines and on Boat?
- **Options:**
  - **A.** Run on the image disk size and warn the customer. Reserve the
    actual disk in scheduling and show both requested and actual sizes.
    Count concurrent jobs with that actual disk in the dialog. Boat uses
    the cheapest size that covers both the label and the image. This keeps
    jobs running but changes the promised resource size. Supply the image
    minimum to the control plane as described above.
    Effort: Medium.
  - **B.** Refuse an undersized label in settings. State the minimum and
    explain how to increase the disk. If an image outgrows an existing
    label, stop new starts for that label until an Admin increases its
    disk. Show the required minimum. This adds settings rejection and can
    pause jobs. Supply the image minimum to the control plane.
    Effort: Medium.
  - **C.** Count the image minimum in placement. A smaller label fits no
    own machine, so the dialog warns before save. It uses a fitting Boat
    size when enabled, or waits. This preserves the label size but can
    force paid overflow after an image update. Supply the image minimum
    to the control plane. Effort: Medium.
  - **D.** Make label disk mean free job disk. Add the image space to each
    VM root disk and count the actual total in scheduling and Boat size
    selection. This changes the label table, site, mockups, and the
    approved rule that root disk size equals label disk size. It keeps
    one full image and one writable root. Effort: Large.
- **Left out:**
  - Keep only the runtime capacity error: the customer can still save a
    label that the scaler sends to an agent which refuses it.
  - Shrink the software image: this breaks the approved full-software
    requirement.
  - Add a writable layer over a shared read-only image: this replaces the
    approved reflink disk model.
  - A dialog warning alone: the scaler still places jobs that cannot run.
  - Give a larger disk only in the agent: A also keeps scheduling honest.
  - Shrink each job copy: a smaller built image has the same lower limit
    without moving image blocks during every start.
  - Remove the disk field and derive disk from vCPU: D keeps that choice
    while protecting free job space.
- **Wider fix:** D. The disk setting excludes the image, so a larger image
  cannot consume the promised free job disk.
- **Previous examples:** The default label previously requested 60 GB for
  an image that uses 58.3 GB before filesystem costs. The user raised it
  to 80 GB instead of removing software.
- **Related:** `own-machine-storage-measurement.md` records host capacity
  measurements. This decision concerns the logical disk size of each VM.
- **Recommendation:** A, changed from B after the options check. It keeps
  labels usable after image growth, as Boat already uses larger sizes.
  D gives the strongest protection at the cost of a different disk model.
- **Action:** ask the user, because the options change behavior that the
  approved disk-sizing decision does not settle.
