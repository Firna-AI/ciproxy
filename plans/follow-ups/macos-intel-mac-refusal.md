# The install script does not refuse an Intel Mac clearly

- **Severity:** Low. **Effort:** Small. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-23, from `calummoore/support-mac-os-kq1u`, the end of
  the plan `plans/macos-own-machines.md`.
- **Location:** the install script of `futex-ai/cli-release`;
  `crates/ciproxy-agent/src/platform/macos/host.rs`.
- **Reason:** The fix belongs in another repository.

- **Background:** The release has only an `aarch64-apple-darwin` agent.
  The agent refuses a Mac without `hw.optional.arm64`, and the site says
  that a Mac needs Apple Silicon. Decision 13 of the plan chose a refusal
  in the install script, with the docs as the stopgap.
- **Problem:** On an Intel Mac, the install script finds no asset and
  fails with a download error, not with "Apple Silicon is required".
- **Options:**
  - **A.** The install script refuses an x86_64 Mac with a clear line.
    Effort: Small.
  - **B.** Keep the docs only. Effort: none.
- **Recommendation:** A.
- **Action:** a change in `futex-ai/cli-release`.
