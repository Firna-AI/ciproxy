# The Jobs API shows a macOS disk hold as the VM disk

- **Severity:** Medium. **Likelihood:** Common. **Effort:** Large.
  **Confidence:** High. **Harder later:** Yes.
- **Kind:** Fix after merge.
- **Added:** 2026-11-23, from PR #141, branch `calummoore/macos-s-disk-real-use`,
  review finding 4.
- **Location:** `crates/ciproxy-agent/src/launches.rs`, the `RunnerReport`
  of `start`; `crates/ciproxy-control-plane/src/reconciliation.rs`,
  `save_own_size`; `crates/ciproxy-store-pg/src/job_views.rs`;
  `crates/ciproxy-api/src/wire/jobs.rs`.
- **Reason:** A second disk fact for each runner reaches the protocol, the
  store, and the Jobs API.

- **Terms:**
  - **Disk hold:** the job disk that one VM holds in placement. On a Mac it
    is the label's free disk and a 4 GB margin, through `RootRule`.
  - **VM disk:** the logical size of the disk that the VM boots. On a Mac
    it is the size of the image `disk.img`, such as 100 GB.
- **Background:**
  - `RunnerReport::size.disk_bytes` holds the disk hold. The control plane
    saves it as the runner's allocation, and the Jobs API returns it as the
    actual disk size of the job's VM.
  - On Linux the hold is the root length, so the two values agree.
  - On a Mac, a 20 GB label now gives 24 GB, while the VM disk is 100 GB.
    Before PR #141 the Jobs API showed the Linux root of the image, about
    210 GB for a 200 GB image, which was also not the VM disk.
- **Problem:** The Jobs API gives the wrong VM disk for each macOS job.
- **Impact if no action:** Customers and API clients see a disk size that
  the job's VM did not have. Old jobs cannot be corrected later.
- **Root question:** How does ciproxy keep both the disk that a job's VM
  had and the disk that its machine held?
- **Options:**
  - **A.** Define the Jobs API value as the held disk on a Mac, and update
    its contract and the app. Effort: Medium.
  - **B.** Save the image's disk length as a separate job fact, and read it
    in the Jobs API. Effort: Large.
  - **C.** Carry the VM disk length and the hold as two facts of the runner
    allocation, from the agent's report. Handle old records with no
    length. Effort: Large.
- **Left out:** A look-up of the newest image for old jobs: it can give
  the length of another release.
- **Wider fix:** C. Placement and reporting read different, named facts.
- **Previous examples:** Commit `fa5974dd` kept the Linux label request
  apart from the actual disk allocation.
- **Recommendation:** C.
- **Action:** fix after merge, apply C, because it changes the runner
  report, the store, and the Jobs API.
