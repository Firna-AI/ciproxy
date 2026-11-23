# Agent warnings hide the cause of internal errors

- **Severity:** Medium. **Likelihood:** Common. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Wider fix.
- **Added:** 2026-11-23, from the branch
  `calummoore/macos-t-area-disk-access`, which fixed the top-level log.
- **Location:** `crates/ciproxy-agent/src`, each `tracing` call that logs
  an agent `Error` with `%error`, 67 calls at the time. For example
  `reporting.rs`, `controller.rs`, `cache.rs`, and the macOS adapters.
- **Reason:** The branch fixed only the log line of a failed command. The
  other calls are many small changes across the agent.

- **Terms:**
  - **Internal error:** `Error::Internal` in `error.rs`. Its display text
    is only "[agent] internal error". Its context frames and its source
    chain hold the cause.
  - **Diagnostic text:** `error_diagnostic::diagnostic` gives the display
    text, the context frames, and each cause that holds no data. It never
    writes a URL, because an image URL is a signed link.
- **Background:** On the test Mac, the daemon failed every 5 s with only
  "[agent] internal error". The cause was macOS privacy control: "list
  images: Permission denied". `main.rs` now logs the diagnostic text as
  `cause`.
- **Problem:** A warning such as "cache budget unavailable" still logs
  only "internal error" for an internal failure.
- **Impact if no action:** A failure that the agent survives leaves no
  cause in the log, so a person must reproduce it to find the cause.
- **Root question:** How does each agent log line give the safe cause of
  an error?
- **Options:**
  - **A.** Add a `cause = %diagnostic(&error)` field to each call.
    Effort: Small.
  - **B.** Give `Error` a log wrapper type whose display is the
    diagnostic text, and use it in each call. Effort: Small.
- **Left out:** Logging the full `Debug` form, which can hold a signed
  URL.
- **Wider fix:** B, with a check that finds `%error` on an agent `Error`.
- **Previous examples:** The top-level log line in `main.rs`, on the
  branch above.
- **Recommendation:** B. One wrapper keeps the rule in one place.
- **Action:** fix after merge, apply B.
