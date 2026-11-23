# A Mac has no machine cache

- **Severity:** Medium. **Effort:** Large. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-23, from `calummoore/support-mac-os-kq1u`, the end of
  the plan `plans/macos-own-machines.md`.
- **Location:** `crates/ciproxy-agent/src/platform/macos/`
  (`NoMachineCache`); `crates/ciproxy-cache-sidecar`; the release has
  no macOS cache sidecar.
- **Reason:** Decision 10 of the plan: a second pull request.

- **Background:** macOS jobs restore from the GitHub cache. The Mac page
  says "A Mac has no machine cache yet". The cache sidecar is built for
  Linux only.
- **Problem:** Cache restores on a Mac go over the internet and are slower
  than on a Linux machine with its machine cache.
- **Task:** build the cache sidecar for `aarch64-apple-darwin`, give the
  Mac agent a machine cache on its area volume, add it to the release, and
  update the Mac page, the site, and the mockups.
- **Action:** a new pull request.
