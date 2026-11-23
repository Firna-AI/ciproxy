# The macOS area has no measured space policy

- **Severity:** Medium. **Effort:** Large. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-23, from PR #141, branch `calummoore/macos-s-disk-real-use`,
  decision 1 of its review, options B and C.
- **Location:** `crates/ciproxy-agent/src/macos/hardware.rs`,
  `IMAGE_BUDGET_BYTES` and `MINIMUM_AREA_BYTES`;
  `crates/ciproxy-agent/src/platform/macos/area.rs`;
  `crates/ciproxy-agent/src/platform/macos/images.rs`.
- **Reason:** A policy from the measured image size changes the install,
  the image staging, and the job starts together.

- **Terms:**
  - **Area volume:** the APFS volume where the Mac agent keeps images and
    VM clones. Its default quota is three quarters of the free container
    space, at least 132 GB.
  - **Image budget:** the part of the area that the job disk count leaves
    for images: 100 GB by default on a Mac.
- **Background:**
  - On 23 November 2026, the parent session chose a 100 GB default budget
    and a 132 GB minimum quota for the standard image: a 100 GB disk with
    77.2 GB allocated, 21.8 GB free for a job, and a 46.2 GB `macos.tar`.
  - A first import takes about 123 GB at the peak (download and image). An
    update needs the new download and image beside the old image, about
    200 GB in all. A Mac near the minimum quota cannot hold that.
  - The agent refuses a new VM when the real free space of the area is
    short, so an update does not let new jobs fill the volume.
- **Problem:** On a small Mac, an image update can fail for lack of space,
  and the fixed budget does not follow the size of a new image.
- **Impact if no action:** A small Mac can stay on an old image, or its
  update can fail, when a new release comes.
- **Root question:** How does a Mac with a small area keep one image, one
  default job, and room for an update?
- **Options:**
  - **A.** Keep the fixed values, and document the area that an update
    needs. Effort: Small.
  - **B.** A small-area policy: keep one verified image and one default
    job, remove the old image before an update when space is short, and
    hold the update while jobs run. Effort: Large.
  - **C.** One policy for install, image staging, and job starts, from the
    measured allocated size and archive size of each release. Effort: Large.
- **Left out:** A larger default quota: many 256 GB Macs do not have the
  free space.
- **Wider fix:** C.
- **Previous examples:** Commit `3602a4b6` aligned the Linux storage
  budget, hardware minimum, examples, and capacity tests.
- **Recommendation:** C, after the 100 GB standard image has run on the
  test Mac.
- **Action:** fix after merge, apply C.
