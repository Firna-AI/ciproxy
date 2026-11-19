# Live smokes time out on a cold image

- **Severity:** High. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-18, from `calummoore/ci-proxy-load-balancer`, live
  run 5, item 4 of its report.
- **Location:** `crates/ciproxy-agent/scripts/smoke-common.sh`, functions
  `prerequisites`, `wait_for_local_job`, and `fail`.
  `crates/ciproxy-agent/scripts/smoke-host.sh`.
  `crates/ciproxy-agent/scripts/own_smoke.py`, methods `Smoke.__init__` and
  `Smoke.wait`.
- **Reason:** The user chose E on 19 November 2026, as work after the
  merge.

- **Terms:**
  - **Cold image:** the state of a machine that has no verified job image
    yet, because its agent has not finished the first download.
  - **Live smoke:** a script under `crates/ciproxy-agent/scripts/` that runs
    real GitHub jobs on a real machine. Seven exist.
  - **Fresh-machine smoke:** `smoke-host.sh`. It installs the agent on a new
    host with `/var/lib` on ext4, repairs the install, runs two real jobs,
    and uninstalls.
  - **Firewall proof:** `smoke-cache-egress.py`. It checks the firewall of
    the machine cache service, its protected starts, and real cache jobs,
    before and after a reboot.
  - **Image index:** `/var/lib/ciproxy/area/images/index.json`. The agent
    lists the verified images there, and `newest` names the image of new
    VMs.
- **Background:**
  - After install, the agent downloads, checks, and extracts the selected
    job image release. Then it writes the release to the image index.
  - The control plane routes a job to a machine only when the report of the
    machine names a selected image. `connected` in `machine_input` of
    `crates/ciproxy-control-plane/src/snapshot.rs` holds this rule.
  - A machine that already has an image keeps it during the download of a
    newer release. So only a cold image blocks the jobs.
  - Release `2026-11-18-6ed25c36` has a Firecracker archive of
    17,994,882,421 bytes. Its root disk is 60,263,759,872 bytes.
  - The shell smokes start in `prerequisites` of `smoke-common.sh`. The
    Python smokes start in `Smoke.__init__` of `own_smoke.py`.
  - On 17 November 2026, the user chose to keep the fresh-machine smoke as
    a check before the merge into `main`.
- **Problem:**
  - On 18 November 2026, the fresh-machine smoke ran on a new ext4 VM with
    agent 0.1.3 and release `2026-11-18-6ed25c36`.
  - The install, the repair, and the shared reflink check passed.
  - The first job then waited for the image download. `wait_for_local_job`
    polls 180 times with a pause of 1 second, so its guard ends after about
    3 to 9 minutes. The smoke failed with this output:

    ```text
    FAIL: the job did not appear in this machine state; check staging routing and its guest image
    ```

  - The two reflink jobs and the uninstall check did not run.
  - The firewall proof failed in the same way on a cold image. Its job
    window of 600 seconds, the default of `Smoke.wait`, ended during the
    download. Its rerun on the prepared image passed before and after a
    real reboot.
  - Each other live smoke meets the same failure when it runs first on a
    new host. `crates/ciproxy-agent/smoke-checks.md` asks for a fresh
    disposable host for the own-machine runtime check.
- **Impact if no action:** The fresh-machine smoke cannot pass on a new
  host, so the merge into `main` stays blocked. Each live smoke that runs
  first on a new host meets the same failure.
- **Root question:** How does a live smoke know when a new machine is ready
  to take its first job?
- **Options:**
  - **A.** Add an image readiness wait to the fresh-machine smoke only. It
    waits until the machine has the selected image, with its own time limit
    and its own failure text. Then the unchanged job guard starts. The
    other smokes keep the failure. Effort: Small.
  - **B.** Add the readiness wait to the fresh-machine smoke and the
    firewall proof. Both keep their job guards unchanged. The other five
    smokes keep the failure. Effort: Medium.
  - **C.** Make `ciproxy-agent install` wait until the selected image is
    ready before it returns. The customer install then lasts as long as the
    download. This changes the product. Effort: Medium.
  - **D.** Raise the job guards of both smokes to cover the download. A slow
    job start then hides inside the download time, and the failure text
    cannot name the phase. Effort: Small.
  - **E.** Put the readiness wait in the shared entry of every live smoke:
    `prerequisites` and `Smoke.__init__`. It reads the image index until
    `newest` names an image in `available`, the rule that routing uses. It
    runs before the first warm wait and the first dispatch, with its own
    limit and failure text. Every job guard stays unchanged. Each smoke
    then reads the internal index format. Effort: Medium.
- **Left out:**
  - A host with a preloaded image does not prove the first download of a
    fresh install, which the smoke must prove.
  - A smaller image breaks the rule of `docs/protocol/agent-images.md` that
    the image holds the software of the `ubuntu-latest` runner image.
  - A new journal event for a published image is worse than the image index
    on every point. A smoke can miss an event from before its start.
  - A faster download fails the goal. A slow host link still passes a fixed
    job guard, and the failure text still cannot name the phase.
  - A phase from the GitHub job state fails the goal. A queued job cannot
    tell the download apart from a routing fault.
- **Wider fix:** E. Every live smoke calls its shared entry first, so no
  smoke starts its job timer on a cold image. C also prevents the class,
  also for customers, but it changes the product.
- **Previous examples:** On 18 November 2026, the first attempt of the
  firewall proof failed on the cold image, and its rerun on the prepared
  image passed.
- **Related:**
  - `plans/follow-ups/repeat-install-image-download.md`: same cause of a
    longer wait. A repeat install restarts the download, so the readiness
    wait must run after the repair step of the fresh-machine smoke.
  - `plans/follow-ups/machine-image-preparing-status.md`: same cold image.
    The dashboard shows such a machine as healthy.
- **Recommendation:** E, changed from B. The cold image meets every live
  smoke on a new host, and E costs the same as B, which also needs both the
  shell and the Python layer.
- **Action:** fix after merge, apply E, because the user chose E on
  19 November 2026, as work after the merge. Until then, the
  fresh-machine smoke cannot pass on a new host.

The fix needs three more details:

- When a smoke names an expected release, the wait must wait for that
  release. On a host with an older image, a smoke can otherwise pass on the
  older image while the new one downloads.
- The wait must run before the first dispatch. With a label that allows
  Boat, a job that waits in the queue during the download starts on Boat.
  `place_boat` in `crates/ciproxy-scaler/src/overflow.rs` does not wait
  when no connected own machine fits.
- `fail` in `smoke-common.sh` leaves the dispatched GitHub run in the
  queue, so a later run can find a stale job. The shared entry can cancel
  that run when the smoke fails.
