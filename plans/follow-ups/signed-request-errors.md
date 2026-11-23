# Signed image request errors can hold their URL

- **Severity:** High. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-23, from the branch
  `calummoore/h3-macos-release-from-r2-qjch`, review round 1, finding 2.
- **Location:** `crates/ciproxy-control-plane/src/image_release.rs`,
  `without_url`; `crates/ciproxy-agent/src/image_download.rs`, the send
  and status errors of the Linux image download;
  `crates/ciproxy-agent/src/resumable_download.rs`, the `without_url`
  calls.
- **Reason:** The wider fix spans two crates and code that the branch
  that found it does not change.

- **Terms:**
  - **Signed URL:** an image URL that carries its own read credential: a
    Cloud CDN signature for Linux, or an S3 presigned query for macOS.
- **Background:** reqwest 0.13.5 attaches the request URL to send errors,
  status errors, and body decode errors. The `Debug` form of
  `InternalError` from the `internal-error` crate prints the `Display` of
  each source, and the reqwest `Display` ends with `for url (...)`. The
  `Secret` type in `ciproxy-protocol` says that a URL must not enter
  `Debug`, and the hosted `Error` of the control plane says that it never
  holds a credential.
- **Problem:** The control-plane release reader now drops the URL from
  send and JSON errors, with a test in
  `crates/ciproxy-control-plane/tests/macos_release_tests.rs`. Each other
  caller still decides for itself. The Linux agent download in
  `image_download.rs` keeps the signed CDN URL in its send and status
  errors. The resumable macOS download in `resumable_download.rs` calls
  `without_url()`, but no test fails when that call goes.
- **Impact if no action:** A log line that prints an error with `?` can
  publish a URL that reads an image for up to 2 hours.
- **Root question:** How does each failed signed request report what
  failed without its credential?
- **Options:**
  - **A.** Call `without_url()` in `image_download.rs`, and add one test
    for each download that checks the `Debug` text of a failed send.
    Effort: Small.
  - **B.** Add one signed-request type that takes the `Secret` URL, sends
    the GET, and returns errors without the URL for the send, the status,
    and the body. Move the reader and both agent downloads to it. Effort:
    Medium.
- **Left out:** A redaction layer in the log subscriber: the URL stays in
  the error value, and the agent does not use the hosted subscriber.
  Header signing for the control plane's own reads: it leaves the agent
  downloads, which must use a URL.
- **Wider fix:** B. No caller gets a raw reqwest error for a signed URL.
- **Previous examples:** Commit `703b9b19` checked only successful signed
  CDN reads.
- **Recommendation:** B. It removes the case at each signed request.
- **Action:** fix after merge, apply B, because it changes agent code that
  the branch that found it does not touch.
