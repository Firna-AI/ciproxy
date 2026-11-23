# A job near its memory limit can hang

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-23, from the branch `calummoore/swap-support-3dll`,
  a problem that the options check of its plan found.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-agent/src/vm_liveness.rs:1` and
  `image/ciproxy-guest.service`.
- **Reason:** The swap change adds headroom, and how the agent finds a
  stalled job is a separate choice.

- **Terms:**
  - **zram:** a Linux block device that keeps its data compressed in
    memory. The branch above adds it as swap to each Linux job VM.
  - **Thrashing:** a state in which the kernel spends most of its time
    moving pages in and out of swap, so the processes make almost no
    progress.
  - **PSI:** pressure stall information, the kernel's measure of the time
    that processes wait for memory, CPU, or I/O.
- **Background:**
  - Each Linux job VM gets swap in memory, sized at its memory and capped
    at 8 GiB. With swap, the kernel stops a process for lack of memory
    later than without swap.
  - The agent checks only whether the process that owns a VM still runs
    (`crates/ciproxy-agent/src/vm_liveness.rs:1-5`).
  - The job VM kernel has `CONFIG_PSI=y` and
    `CONFIG_PSI_DEFAULT_DISABLED=y`, so PSI needs `psi=1` on the kernel
    command line.
  - When the kernel stops a process of a step, systemd stops the whole
    guest service, because `image/ciproxy-guest.service` sets no OOM
    policy.
- **Problem:** A job that uses close to its memory and its swap can
  thrash for a long time before the kernel stops a process. Nothing in
  the agent or the guest sees this, so the job can hang until its GitHub
  timeout.
- **Impact if no action:** A customer's job can run for up to the job
  timeout and use a slot for that time, instead of a fast failure.
- **Root question:** How does a job that has run out of memory end fast
  and with a clear reason?
- **Options:**
  - **A.** Keep the current behaviour. The job ends at the GitHub
    timeout or when the kernel stops a process. Effort: Small.
  - **B.** Turn on PSI with `psi=1` and start `systemd-oomd` in the job VM
    for the control group of the job. It stops the job's processes after
    a set time of high memory pressure. Cost: a new stop rule that can end
    a slow job that would pass. Effort: Medium.
  - **C.** Turn on PSI, and let the guest program read the memory
    pressure and report it to the control plane with its samples. The job
    page then names memory pressure as the likely cause. Cost: no fast
    end, only a clearer reason. Effort: Medium.
- **Left out:** A smaller swap, because it gives less headroom and only
  shortens the time of thrashing.
- **Wider fix:** None. The out-of-memory decision of the plan
  `job-step-cpu-and-memory.md`, on the branch
  `calummoore/action-cpu-and-memory-usage`, decides what happens to the
  guest when a process stops.
- **Previous examples:** None found.
- **Recommendation:** C. It gives the customer the reason without a new
  stop rule. Choose B for a fast end.
- **Action:** ask the user, because B adds a rule that stops jobs, and no
  requirement covers memory pressure.
