# Usage reports have unstated size limits

- **Severity:** Medium. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-23, from the branch
  `calummoore/action-cpu-and-memory-usage`, review round 2, the first two
  new problems of the check of finding 508.
- **Location:** `crates/ciproxy-guest/src/vsock.rs:77-89`,
  `crates/ciproxy-agent/src/guest_transport.rs:103`,
  `crates/ciproxy-guest/src/boat.rs:41`,
  `crates/ciproxy-control-plane/src/boat_callback.rs:26`,
  `crates/ciproxy-protocol/src/usage.rs:16`, and
  `docs/protocol/job-usage.md:160-163`.
- **Reason:** Each option states a size rule for usage reports that no
  requirement covers, and the recommended option also changes Boat
  behaviour beyond the contract.

- **Terms:**
  - **Usage report:** the CPU and memory samples and the steps of a job.
    The guest, the ciproxy program in the job VM, sends them in its exit
    report, and on Boat also in a sample report every 30 seconds.
  - **Exit line:** the one JSON line in which the guest of an own machine
    reports the runner exit to the agent over vsock.
  - **Boat route:** the control-plane route `/hooks/boat/runner-exit`,
    which takes the exit report and the sample reports of a Boat sandbox
    (`crates/ciproxy-control-plane/src/boat_callback.rs:17-21`).
- **Background:**
  - The points of a usage report stay small.
    `docs/protocol/job-usage.md:63` says that the samples of a job of 6
    hours stay under about 100 KB.
  - The steps can add far more. The guest sends at most 1,000 steps, each
    with a name of at most 256 characters
    (`docs/protocol/job-usage.md:160-161`,
    `crates/ciproxy-protocol/src/usage.rs:16` and `:18`). The JSON escapes
    make a name of other characters much larger than 256 bytes.
  - Measured sizes of compact JSON reports in the shape of the wire
    snapshot:
    - 3,600 points on a VM with 2 vCPU: 43 KB.
    - 3,600 points on a VM with 64 vCPU: 54 KB.
    - 500 steps with ASCII names of 220 characters, steps only: 153 KB.
    - 3,600 points and 1,000 ASCII names of 256 characters: 384 KB.
    - The same with names of emoji: 1.16 MB.
    - The same with names of control characters: 1.68 MB.
  - On an own machine, the guest drops every step of an exit line that
    would reach the line limit of 1 MiB of the agent's guest reader, and it
    keeps the points (`crates/ciproxy-guest/src/vsock.rs:13` and `:77-89`,
    `crates/ciproxy-agent/src/guest_transport.rs:103`). The test at
    `crates/ciproxy-guest/src/_tests_/vsock_tests.rs:12` shows the drop with
    1,000 names of 256 emoji.
  - The design call "Oversized steps" of this branch gives the reason: the
    agent would otherwise refuse the whole report. No protocol doc states
    the rule. `docs/protocol/job-usage.md:84-85` names only a job whose
    steps the guest cannot read.
  - On Boat, the guest posts each report with `BoatCallback::post`, with no
    size check (`crates/ciproxy-guest/src/boat.rs:41-52`). The route reads
    each report through the `Json` extractor of axum
    (`crates/ciproxy-control-plane/src/boat_callback.rs:26`).
  - No layer sets a body limit on that route, so the default of axum
    applies: 2,097,152 bytes in `axum-core` 0.5.6. Only the GitHub webhook
    route raises its limit, to 25 MiB
    (`crates/ciproxy-control-plane/src/http.rs:37`). No doc states the
    limit of the Boat route, and no test checks it.
  - So the same job can keep its steps on Boat and lose them on an own
    machine.
  - The guest limits allow a Boat report of about 1.7 MB, under the 2 MiB
    default, so no report fails today. A lower body limit later, for
    example from an axum upgrade or a new layer, would refuse such a
    report, also an exit report, with no failed test.
  - The agent forwards an own-machine exit to the control plane through
    the gateway, whose message limit is 16 MiB
    (`crates/ciproxy-gateway/src/transport.rs:43`).
- **Problem:**
  - Two size limits decide what a usage report keeps, and no doc states
    either of them. The guest of an own machine drops all steps at 1 MiB.
    The Boat route refuses a body over 2 MiB, by a framework default that
    no test pins.
  - The two paths treat the same report in two ways.
- **Impact if no action:** A job with long names of other characters loses
  all its steps on an own machine, and a reader of the docs cannot learn
  why. A lower body limit on the Boat route would refuse reports, also exit
  reports, and no test would fail.
- **Root question:** What is the largest usage report that each path
  accepts, what does the guest do with a larger one, and which doc states
  it?
- **Options:**
  - **A.** State the two limits as they are.
    - Add to Checks of `docs/protocol/job-usage.md` that the guest of an
      own machine drops the steps of an exit line that would reach the
      line limit of 1 MiB, and keeps the points.
    - State in `docs/protocol/overflow-sandbox-end.md` that the Boat route
      accepts a body of at most 2 MiB. Set that limit with an explicit
      `DefaultBodyLimit` on the route. Add a route test with a report just
      under the limit and one over it.
    - Cost: the two paths keep two rules, so a job can keep its steps on
      Boat and lose them on an own machine.
    - Effort: Small.
  - **B.** Make the step limits keep every report under 1 MiB, so that no
    report needs a drop.
    - Add a limit of encoded bytes for each name beside the limit of 256
      characters, so that 1,000 steps and 3,600 points stay under 1 MiB.
      For example, 512 bytes of JSON for each name.
    - The guest cuts a longer name, and the control plane drops the steps
      of a report that passes the new limit.
    - Cost: a name of other characters gets cut before 256 characters, and
      the step limits at `docs/protocol/job-usage.md:160-161` change.
    - Effort: Medium.
  - **C.** Raise the line limit of the agent's guest reader above the
    largest report, for example to 2 MiB, and state both limits.
    - The guest of an own machine then keeps every report that the step
      limits allow, as Boat does today.
    - Cost: the agent reads larger lines from a VM that job code controls.
      The agent runs on the machines of the customer, so an old agent
      still refuses a large line. The guest must keep its drop until each
      agent has the new limit.
    - Effort: Medium.
  - **D.** Make 1 MiB the rule for every usage report.
    - State in Checks of `docs/protocol/job-usage.md` that a usage report
      holds at most 1 MiB, and that the guest drops the steps of a larger
      report and keeps the points.
    - Put the limit beside `MAX_STEPS` in
      `crates/ciproxy-protocol/src/usage.rs`.
    - Move the drop at `crates/ciproxy-guest/src/vsock.rs:77` into one
      guest helper. Call it also before the Boat post at
      `crates/ciproxy-guest/src/boat.rs:41`. Add a test at each sender.
    - `docs/protocol/overflow-sandbox-end.md` then says that a report holds
      at most 1 MiB. Add a route test that a report of 1 MiB passes, so
      that a lower body limit fails a test.
    - Cost: a Boat report between 1 MiB and about 1.7 MB then loses its
      steps, which it keeps today. So D changes Boat behaviour beyond the
      contract.
    - Effort: Medium.
- **Left out:**
  - A cap at 100 KB, because a report at the step limits holds about
    384 KB with ASCII names, and the step limits allow that report.
  - Step deltas in the sample reports, because each sample report holds
    all the usage so far, so that a report never depends on an earlier
    one (`docs/protocol/overflow-sandbox-end.md:68-70`).
  - Steps only in the exit report, because a lost exit report then loses
    every step. On 23 November 2026, the user chose sample reports every
    30 seconds and a hold of the Boat delete, so that a lost exit report
    loses at most the last 30 seconds.
  - A compressed body, because names that do not compress still pass any
    limit, and it adds a decompression path to a hosted route.
- **Wider fix:** D. One limit on the protocol type, which both senders use,
  keeps the two paths from drifting apart again.
- **Previous examples:** None found.
- **Related:**
  - `plans/follow-ups/boat-exit-report-response-log.md`, the same code,
    for D only. D adds a step drop before the shared `BoatCallback::post`,
    which that item describes. The drop then applies to both report kinds,
    and that item must say so.
  - `plans/follow-ups/job-usage-step-links.md` and
    `plans/follow-ups/job-usage-short-last-point.md`, the same format.
    Options B and F of the first item add a field to each step, and option
    C of the second adds times to each point. Each raises the size of a
    full report, so check the bound of the chosen option again after them.
- **Recommendation:** D. It gives both paths one stated rule, on the
  protocol type that both senders use, at the limit that the guest of an
  own machine already enforces, and every report keeps its points. Choose
  B if no report may lose its steps. A is the smallest change if the two
  rules may stay apart.
- **Action:** ask the user, because each option states a size rule for
  usage reports that no requirement covers, and D also changes Boat
  behaviour beyond the contract.
