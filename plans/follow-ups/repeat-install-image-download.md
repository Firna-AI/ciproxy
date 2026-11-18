# A repeat install restarts the image download

- **Severity:** Low. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-18, from `calummoore/ci-proxy-load-balancer`, the
  options check of the live run 5 items.
- **Location:** `crates/ciproxy-agent/src/system_service.rs`, function
  `install`. `crates/ciproxy-agent/src/image_download.rs`.
  `crates/ciproxy-agent/src/image_files.rs`, the reclaim of unused image
  files.
- **Reason:** Only an agent restart during the first download meets it,
  and it costs only time.

- **Background:**
  - Each run of `ciproxy-agent install` calls
    `systemctl try-restart ciproxy-agent.service`, in `install` of
    `crates/ciproxy-agent/src/system_service.rs`.
  - A read of the code found that the restarted agent does not continue a
    partial image download. It starts the download again. Nobody measured
    this on a host.
  - The first download of release `2026-11-18-6ed25c36` moves
    17,994,882,421 bytes.
- **Problem:** A repeat install during the first download, such as a
  repair, starts the download again. A crash or a reboot of the agent does
  the same.
- **Impact if no action:** The first jobs of a new machine wait longer. The
  fresh-machine smoke `crates/ciproxy-agent/scripts/smoke-host.sh` repairs
  its install during the download, so it meets this on each run.
- **Root question:** How does a new machine keep its download progress
  across an agent restart?
- **Options:**
  - **A.** Skip the agent restart in a repeat install when the binary and
    the unit file did not change. A restart for another cause still loses
    the progress. Effort: Small.
  - **B.** Continue a partial download after a restart with an HTTP range
    request. Check the full digest at the end, as now. Effort: Medium.
- **Left out:** Only two places exist: the restart is avoided, or the
  download survives it.
- **Wider fix:** B. It keeps the progress for each cause of a restart.
- **Previous examples:** None found.
- **Related:** `plans/follow-ups/smoke-cold-image-readiness.md`: same cause
  of a longer wait. The readiness wait of the live smokes must run after
  the repair step.
- **Recommendation:** B. It covers a repair, a crash, and a reboot.
- **Action:** fix after merge, apply B, because only an agent restart
  during the first download meets it, and it costs only time.
