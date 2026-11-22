# Linux job VMs have no swap

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-22, from the branch
  `calummoore/action-cpu-and-memory-usage`, a problem that the options
  check of its plan found.
- **Location:** `image/provision.sh:56` and `image/boat_build_native.py:30`.
- **Reason:** The change that found it measures the memory of each job
  step, and swap is a separate choice about the memory of each job VM.

- **Terms:**
  - **Swap:** disk space, or compressed memory, where the kernel moves
    memory pages that a process has not used for a while.
  - **Job VM:** the fresh Firecracker VM on an own Linux machine that runs
    one job.
- **Background:**
  - The machine image of a Linux job VM comes from the recipe of the
    GitHub runner images.
  - On Azure, the Azure agent `walinuxagent` makes the swap of that recipe.
    `image/provision.sh:56` disables `walinuxagent`, and no other file of
    the machine build makes swap.
  - The Boat build removes the Azure swap setup, because Boat owns the
    swap of its sandboxes (`image/boat_build_native.py:30`).
  - Public reports say that GitHub-hosted Linux runners have a 4 GB swap
    file.
- **Problem:** A Linux job VM has no swap. So a job that fits in the
  memory of a GitHub-hosted runner can run out of memory on a ciproxy label
  with the same memory.
- **Impact if no action:** A customer who moves a memory-heavy job to
  ciproxy can see it fail, and then needs a larger label.
- **Root question:** How much memory headroom does a Linux job VM give a
  job, compared with a GitHub-hosted runner of the same size?
- **Options:**
  - **A.** Keep no swap. The label docs on the site say that a ciproxy VM
    has no swap, so a memory-heavy job may need the next size. Effort:
    Small.
  - **B.** Add compressed memory swap, such as zram, to the machine image.
    It needs no disk. Cost: CPU time for the compression, and less
    headroom than a 4 GB swap file. Effort: Medium.
  - **C.** Add a 4 GB swap file to the root disk of each job VM, as on a
    GitHub-hosted runner. Cost: 4 GB more disk for each job VM, which
    changes the disk rule of a slot in `docs/protocol/job-routing.md`.
    Swap on a disk that several VMs share also slows the other jobs.
    Effort: Large.
- **Left out:** A swap file on the host for each VM, because Firecracker
  gives a VM only its own drives.
- **Wider fix:** None. This is one setting of the image.
- **Previous examples:** None found.
- **Recommendation:** B. It gives headroom close to a GitHub-hosted runner
  without a change to the disk rule.
- **Action:** ask the user, because each option changes the memory or the
  disk that a job VM gives a job, and no requirement covers swap.
