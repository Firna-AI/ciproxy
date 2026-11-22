# A plain HTTP macOS prefix sends a signed URL

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-23, from the branch
  `calummoore/h3-macos-release-from-r2-qjch`, review round 1, finding 3.
- **Location:** `crates/ciproxy-control-plane/src/macos_release.rs`,
  `MacosImageSettings::source`;
  `crates/ciproxy-control-plane/src/application.rs`, `release_client`.
- **Reason:** Each fix that refuses a prefix adds a new rejection, and
  the always-on rules reserve that choice for the user.

- **Terms:**
  - **`MACOS_IMAGE_URL_PREFIX`:** the GitHub environment variable that
    gives the R2 endpoint and bucket in path style, for example
    `https://785233830ca9dfa2eb7c3561dd52f675.r2.cloudflarestorage.com/ciproxy-macos-images`.
- **Background:** The control plane signs its reads of the macOS
  `manifest.json` and `downloads.json` with a presigned query that is
  valid for 5 minutes. Its release client follows no redirect and accepts
  `http://`. The Mac agent refuses an archive URL that is not `https://`.
  The Linux CDN read uses the same client, and deploy builds its prefix
  with `https://`.
- **Problem:** When an operator sets the prefix with `http://`, the first
  signed manifest URL goes out in plaintext. R2 answers with a 301 to
  HTTPS, which the client does not follow, so macOS images stay off.
- **Impact if no action:** A network observer can read one release-facts
  file for 5 minutes after a misconfigured start. No archive URL leaks.
- **Root question:** How does a wrong scheme in a release prefix never
  send a signed URL in plaintext?
- **Options:**
  - **A.** Check the scheme where `MacosImageSettings` builds its source.
    Accept `https://`, and `http://` only to a loopback host for tests.
    Any other prefix turns macOS images off with a warning that names
    `MACOS_IMAGE_URL_PREFIX`. Adds a rejection. Effort: Small.
  - **B.** Build the prefix in deploy from the fixed account ID and bucket
    and remove the GitHub variable. Adds no rejection, but changes the
    recorded settings and does not cover a start outside deploy. Effort:
    Medium.
  - **C.** Make the shared release client HTTPS-only for both platforms,
    with TLS in the test stores. Adds a rejection, also for Linux. Effort:
    Medium.
- **Left out:** An egress policy that blocks port 80: a cluster-wide
  allow list for one setting, and the error would not name the setting.
- **Wider fix:** C keeps every signed release read on TLS.
- **Previous examples:** The agent's HTTPS-only image client in
  `crates/ciproxy-agent/src/application_images/client.rs`.
- **Recommendation:** A. It stops the request before it leaves and names
  the setting, for a Small change.
- **Action:** ask the user, because each option except B adds a new
  rejection of a setting, and only the user adds a rejection.
