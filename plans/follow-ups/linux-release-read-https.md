# The Linux release read accepts plain HTTP

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-23, from the branch
  `calummoore/h3-macos-release-from-r2-qjch`, review round 1, check of
  finding 3.
- **Location:** `crates/ciproxy-control-plane/src/application.rs`, the
  Linux `SignedReleases` in `Production::serve`;
  `crates/ciproxy-control-plane/src/config.rs`, `IMAGE_URL_PREFIX`.
- **Reason:** The Linux read and its setting are older than the branch
  that found this, and a fix adds a new rejection.

- **Terms:**
  - **`IMAGE_URL_PREFIX`:** the pod setting that names the Cloud CDN
    `/images` origin of the Linux image. Deploy builds it with
    `https://`.
- **Background:** At start, the control plane signs its reads of the
  Linux `manifest.json` and `downloads.json` with the Cloud CDN key. The
  signature is valid for 2 hours. The macOS prefix must start with
  `https://` since 23 November 2026, and a wrong macOS prefix turns macOS
  images off with a warning. The Linux prefix has no such rule.
- **Problem:** A manual `IMAGE_URL_PREFIX` with `http://` sends the signed
  manifest URL in plaintext. Deploy never sets such a value.
- **Impact if no action:** A misconfigured manual start can expose a
  2-hour read of one Linux release file.
- **Root question:** How does every signed Linux release read use TLS?
- **Options:**
  - **A.** Require `https://` for `IMAGE_URL_PREFIX` at start, as a
    missing setting does today. Adds a rejection. Effort: Small.
  - **B.** Make the shared release client HTTPS-only for both platforms,
    and serve the executable test origin over TLS. Adds a rejection.
    Effort: Medium.
  - **C.** Change nothing, because deploy always builds an `https://`
    prefix. Effort: Small.
- **Left out:** A redirect to HTTPS: the signed request line goes out
  before the answer.
- **Wider fix:** B keeps every signed release read on TLS.
- **Previous examples:** The macOS prefix rule in
  `crates/ciproxy-control-plane/src/macos_release.rs`.
- **Recommendation:** A. It matches the macOS rule for a Small change.
- **Action:** ask the user, because the fix adds a new rejection of a
  setting.
