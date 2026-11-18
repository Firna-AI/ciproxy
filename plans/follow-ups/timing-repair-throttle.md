# Timing repair ignores GitHub throttle responses

- **Severity:** Medium. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`, a problem that the check of
  finding 1005 found.
- **Source:** Review.
- **Category:** Performance.
- **Location:** `crates/ciproxy-github/src/timing.rs`,
  `RepairTiming::repair`, `crates/ciproxy-github/src/rest/job_times.rs`,
  `RestClient::times`, and the request helpers of `RestClient` in
  `crates/ciproxy-github/src/rest/`.
- **Reason:** The repair already backs off each attempt, so only a large
  backlog keeps sending lookups after a throttle response.

- **Terms:**
  - **Timing repair:** `RepairTiming::repair`, which reads the real start
    and finish of a job from the GitHub Actions API. It runs in the
    accounting worker.
  - **Throttle response:** an HTTP 403 or 429 reply of GitHub for its
    primary or secondary rate limits.
  - **Installation token:** the GitHub token of one App installation. The
    timing repair and other reads of the same installation share its
    limits.
- **Background:**
  - The accounting worker runs the timing repair for each workspace, then
    the metering refresh. The ten-second tick signals it, and its passes
    never overlap.
  - `repair` loops over the pending attempts of a workspace. For each one,
    it calls `job_times`, which reads
    `repos/{repository}/actions/runs/{run_id}/jobs` with the installation
    token.
  - On any error, it logs "real job timing lookup will retry" and goes on
    to the next attempt in the same pass.
  - Each attempt has its own backoff: 30 seconds, doubled after each miss,
    up to 300 seconds.
  - No adapter in `ciproxy-github` reads `retry-after` or `x-ratelimit-*`.
  - GitHub asks a client to wait as `retry-after` says. When
    `x-ratelimit-remaining` is 0, it asks the client to wait until
    `x-ratelimit-reset`. Otherwise it asks the client to wait at least one
    minute.
  - The same installation token also serves the run facts of each job and
    the GitHub cache entry list.
  - On 17 November 2026 the final review chose to make the webhook
    recovery obey these throttle responses. That fix does not reach the
    timing repair.
- **Problem:** After GitHub throttles an installation token, the timing
  repair goes on to the next attempt with the same token, and it reads no
  throttle header.
- **Impact if no action:** A large backlog of pending attempts keeps sending
  lookups while GitHub throttles the token. That extends the throttle, and
  it delays the other reads of the same installation, such as the cache
  entry list.
- **Root question:** How do the GitHub reads of ciproxy stop after a
  throttle response until GitHub allows them again?
- **Options:**
  - **A.** In `repair`, treat a 403 or 429 as a throttle response when it
    has `retry-after`, or when `x-ratelimit-remaining` is 0. Stop the pass
    for that installation, and skip it until the given time, or for at
    least one minute. Add a test with a fixed clock. Effort: Small.
  - **B.** Give each GitHub credential one throttle state in
    `ciproxy-github`.
    - Send the requests of `HttpDeliveries`, `AppInstallationClient`,
      `RotatingTokens`, and `RestClient` through one shared helper.
    - The helper reads the throttle responses for each credential: the App
      JWT and each installation token. It holds or refuses the later
      requests of that credential until the reset time.
    - Keep the schedule of the webhook recovery.
    - Cost: one new helper, changes in four adapters, and a composition
      change. The `scaleset` SDK mints its own tokens outside the helper. A
      hold can also delay token minting when GitHub limits only one
      endpoint.
    - Effort: Medium.
  - **C.** Back off the whole accounting worker after a throttle response.
    Cost: it also delays the metering refresh and the repairs of other
    installations. Effort: Small.
- **Left out:** A fixed longer backoff for each attempt: it fails the goal.
  The next pass still sends the due attempts of a large backlog after a
  throttle response.
- **Wider fix:** B. After a throttle response, it holds every GitHub call of
  that credential, so the next GitHub loop cannot repeat this defect.
- **Previous examples:** On 17 November 2026 the final review found that the
  webhook recovery retried every failed event at once, also after a
  throttle response. It fixed that in the branch.
- **Recommendation:** B. One throttle state for each credential covers the
  timing repair and every other GitHub caller. A is the direct fix for the
  timing repair alone.
- **Action:** fix after merge, apply B, because the backoff of each attempt
  already limits the repeats, and B reaches four adapters.
