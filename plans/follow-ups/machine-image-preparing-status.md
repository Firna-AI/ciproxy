# New machines look ready before their image is

- **Severity:** Medium. **Likelihood:** Common. **Effort:** Large.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-18, from `calummoore/ci-proxy-load-balancer`, the
  options check of the live run 5 items.
- **Location:** `ts/app/src/view/machines.ts`, function `machineStatus`.
  `ts/app/src/view/setup-steps.ts`, functions `setupModel` and `firstJob`.
  `ts/app/src/screens/machines/add-machine.tsx`, component `Joined`.
  `docs/mockups/src/components/status.tsx`, the machine states.
  `crates/ciproxy-gateway/src/gateway.rs`, the presence of each machine.
  `crates/ciproxy-control-plane/src/snapshot.rs`, the `connected` field of
  `machine_input`. `docs/protocol/agent.md`, "Install".
  `docs/protocol/job-routing.md`, "Disk".
- **Reason:** The fix adds a machine state that no mockup shows and changes
  what the protocol docs say, and the user approves both.

- **Terms:**
  - **Cold image:** the state of a machine that has no verified job image
    yet, because its agent has not finished the first download.
  - **Presence:** online or offline. The gateway sets it from a contact of
    the agent in the last 60 seconds.
- **Background:**
  - After install, the agent downloads, checks, and extracts the selected
    job image release. Release `2026-11-18-6ed25c36` has a Firecracker
    archive of 17,994,882,421 bytes.
  - The machine API gives `offline` from the presence only.
    `machineStatus` then shows Healthy for an online machine with KVM,
    slots, and no drain.
  - Routing uses a stricter rule. `connected` needs a contact in the last
    60 seconds and a selected image in the report of the machine.
  - The machine API also gives `selected_image_disk_bytes`. It is empty
    until the report names a selected image.
  - The mockups show four machine states: Healthy, Draining, Offline, and
    No slots.
  - Setup and Add a machine show `<host> connected` as a success for each
    machine with KVM. `setupModel` and `firstJob` do not read
    `machineStatus`.
  - The Quickstart says that the first job runs at once when a warm runner
    is ready. A first `ciproxy-agent install` ends with only
    `ciproxy agent installed`.
  - `docs/protocol/agent.md` under "Install" says what the dashboard shows
    after a machine connects. No protocol doc says that a machine without
    an image takes no job.
  - `docs/protocol/job-routing.md` under "Disk" says that configured
    release facts are a fallback until the machine reports its image. But
    the scaler gives such a machine no job.
- **Problem:** During the first image download, the dashboard shows a new
  machine as Healthy and Setup shows it as connected, but routing gives it
  no job. No page says why the first job waits. On 18 November 2026, the
  first job on a new host waited for the whole download.
- **Impact if no action:** A new customer sees a healthy machine while the
  first jobs wait, and no text gives the reason. With a label that allows
  Boat, those jobs start on Boat and cost money.
- **Root question:** How does a customer know, wherever they wait for a new
  machine or its first job, that the machine takes no jobs yet, and why?
- **Options:**
  - **A.** Add a Preparing state to the machine list and the machine page,
    for an online machine without a selected image. Its text says that the
    machine gets the job image and that jobs start when it is ready. The
    app takes the state from `selected_image_disk_bytes`. The mockups get
    the state first. Add one sentence about the first download to the "Add
    a machine" guide. Effort: Medium.
  - **B.** Apply A, and show the download progress. The agent report then
    carries the bytes done and the total. This changes the protocol, the
    agent, the API, and the app. Effort: Large.
  - **C.** Add only the sentence about the first download to the "Add a
    machine" guide. The dashboard keeps Healthy. Effort: Small.
  - **D.** Apply A on every page that shows a new machine or its first job.
    Setup and Add a machine say that the machine gets the job image. The
    first-job step of Setup gives the same reason while the job waits. A
    first install ends with one sentence about the download. The Quickstart
    gets the sentence too. `docs/protocol/agent.md` states the new state,
    and `docs/protocol/job-routing.md` states that a machine takes jobs
    only after it reports its image. Setup and Add a machine need mockup
    states, and the install line needs an agent release. The app still
    copies the routing rule. Effort: Large.
  - **E.** Apply D, but take the state from one new field of the machine
    API. One function gives each machine a closed state: offline, waiting
    for its first image, or ready. The gateway uses it for the machine
    view, and `machine_input` uses it for `connected`. It lives where both
    read it, for example on `MachineSnapshot` in `ciproxy-store-interface`.
    This changes Rust, the OpenAPI contract, and the app, and the routing
    tests must keep passing. Effort: Large.
- **Left out:**
  - Offline for such a machine fails the goal, because the agent is
    connected and works.
  - An existing state, such as No slots or Draining, fails the goal,
    because it names a wrong cause.
  - Healthy with only a notice and a slot caption: A is better on every
    point at about the same cost. With Healthy, the slot figures still
    count the machine as able to take jobs.
  - An install that waits for the first image fails the goal alone. The
    machine registers before the download, so the dashboard still shows
    Healthy.
  - A faster or lazily loaded first image fails the goal. The dashboard
    still shows Healthy for the wait that remains.
  - An email or Slack alert when the machine is ready breaks a requirement
    that stays. `docs/protocol/billing.md` says that these alerts come
    later.
  - Changing nothing fails the goal.
- **Wider fix:** E. The dashboard state and `connected` come from one
  function, so they cannot drift. A, B, and D copy the routing rule into
  the app, so a new routing condition would show Healthy again.
- **Previous examples:** `plans/follow-ups/machines-boat-start-status.md`.
  `boatTakesJobs` in `ts/app/src/view/machines.ts` copies the scheduler
  rule `available` of `crates/ciproxy-scaler/src/overflow.rs`, so Machines
  promises Boat starts that routing holds.
- **Related:**
  - `plans/follow-ups/smoke-cold-image-readiness.md`: same cold image. The
    Python live smokes can wait on the field of E.
  - `plans/follow-ups/machines-boat-start-status.md`: same class, an app
    copy of a routing rule.
- **Recommendation:** E, changed from A. It tells a new customer on Setup
  why the first job waits, and its state cannot drift from routing. D is
  the next best when no API change is wanted. No text may promise a time,
  because a slow link makes the download long.
- **Action:** ask the user, because the fix adds a machine state that no
  mockup shows and changes what the protocol docs say, and the user
  approves both.
