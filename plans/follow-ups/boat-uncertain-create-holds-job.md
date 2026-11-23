# An uncertain Boat create can hold a job

- **Severity:** High. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-22, from `conductor/ciproxy-boat-start-ready-deadline`,
  found by its orchestrator while it read the code for the ready deadline of
  launched Boat starts.
- **Location:** `crates/ciproxy-overflow/src/service/execution.rs:94-110`,
  `crates/ciproxy-overflow/src/service/inventory.rs:99-105`,
  `crates/ciproxy-control-plane/src/boat_snapshot.rs:42-62`, and
  `crates/ciproxy-scaler/src/lifecycle.rs:66-88`.
- **Reason:** The ready deadline of that branch covers only launched starts,
  and a bound for a start that never gets a sandbox adds behaviour at a
  boundary that no requirement covers.

- **Terms:**
  - **Uncertain create:** a Boat start in the phase `Creating`, whose create
    request may or may not have reached Boat.
  - **Recovery read:** a read-only lookup at Boat that looks for the sandbox
    of an uncertain create. It never repeats the create request.
  - **Supply:** a runner that the scaler counts for a waiting job, so that
    the job gets no new start.
  - **Ready deadline:** the rule of section "The ready deadline" of
    `docs/protocol/overflow-sandbox-end.md`. ciproxy replaces a launched
    Boat runner that takes no job within 10 minutes of its launch while a
    job waits for it.
- **Background:**
  - ciproxy saves `Creating` before it sends a create request to Boat.
  - Only a fresh create can prove that no create request reached Boat. The
    pinned Boat adapter returns `SandboxCreateNotSent` only for a failure
    before the send. The start then ends as `Refused`
    (`crates/ciproxy-overflow/src/service/preparation.rs:147-158`).
  - When the create call ends in any other error, such as a lost answer,
    the start stays `Creating` (`execution.rs:116-118`). ciproxy then uses
    only recovery reads for it, also after Boat's 24-hour idempotency
    window. A recovery read never proves that no request was sent.
  - The section "What This Crate Does" of `crates/ciproxy-overflow/README.md`
    states both rules.
  - When a recovery read finds no sandbox, the start stays pending
    (`execution.rs:110`), and cleanup skips it (`inventory.rs:105`).
  - No sandbox is known, so no lifetime ends such a start. The same README
    says: "An ambiguous create without a known sandbox gets no inferred end."
  - The control plane counts such a start as a `Starting` Boat runner
    (`boat_snapshot.rs:62`).
  - A waiting job claims an idle starting Boat runner of the same
    organization and label (`lifecycle.rs:66-88`). A starting runner on a
    machine needs a connected machine, but a Boat runner needs no such
    check.
  - The scaler starts a new runner only for a job that claims none
    (`crates/ciproxy-scaler/src/decision.rs:22-43`).
  - The ready deadline counts from the saved launch time, so it does not
    apply to a start that never launched (`lifecycle.rs:32`).
  - GitHub cancels a job that waits 24 hours in the queue for a self-hosted
    runner.
  - Nobody has seen this case. The code read above shows it.
- **Problem:** When a create request never reached Boat, and ciproxy cannot
  prove that, the start stays `Creating` with no end. Its waiting job claims
  it, so ciproxy starts no other runner for that job.
- **Impact if no action:** The job waits until an own machine or another
  runner takes it, someone cancels it, or GitHub cancels it after 24 hours
  in the queue.
- **Root question:** How does a waiting job get another runner when its Boat
  start never gets a sandbox?
- **Options:**
  - **A.** Extend the ready deadline to each Boat start from its dispatch: a
    start that has not launched some minutes after its dispatch is no longer
    supply. The waiting job then gets a new start. The uncertain start keeps
    its recovery reads and its spend bound. A slow but healthy create past
    that time also gets a second start. Effort: Medium.
  - **B.** Count an uncertain create as supply only until its first recovery
    read finds no sandbox. The job then gets a new start at once. A slow
    provider listing can then give a job two starts. Effort: Medium.
- **Left out:**
  - Change nothing, because it fails the goal.
  - A second create request for the same delivery. It breaks the rule of the
    pinned `sandbox-interface` contract and of
    `crates/ciproxy-overflow/README.md` that an uncertain create never sends
    a second create request, because it can start a second paid sandbox.
  - Ending an uncertain create after Boat's 24-hour idempotency window when
    no recovery read found a sandbox. The window starts at dispatch, after
    the job joined the queue. So GitHub has cancelled the job by then, and
    this fails the goal. It would also change the rule that recovery reads
    run forever.
  - An operator command that ends a start that Boat proves it never
    created. Each case then waits until an operator notices it, so it fails
    the goal. `plans/follow-ups/boat-historical-start-recovery.md` holds
    that command.
- **Wider fix:** A. One deadline then covers each Boat start from its
  dispatch to its first job.
- **Previous examples:** `plans/follow-ups/boat-launched-runner-readiness.md`,
  deleted on 22 November 2026, recorded the same class for launched starts:
  a start failure that no report covers strands its job. Its option B, a
  ready deadline for each launched start, became the ready deadline.
  [Its last text][readiness-item] is at commit `dcb9f0b`.
- **Related:** `plans/follow-ups/boat-historical-start-recovery.md`, the
  same cause: a `Creating` start without a sandbox stays pending. That item
  adds an operator command that ends a start that Boat proves it never
  created. This item frees the waiting job without an operator.
- **Recommendation:** A. It reuses the ready deadline and keeps the recovery
  reads of the uncertain start.
- **Action:** ask the user, because a bound for a start that never gets a
  sandbox adds behaviour at a boundary that no requirement covers.

[readiness-item]: https://github.com/Firna-AI/ciproxy/blob/dcb9f0b9bcf368c05a0331170a0ce015c66ab15a/plans/follow-ups/boat-launched-runner-readiness.md
