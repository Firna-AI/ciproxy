# Each macOS image download command hashes the partial file again

- **Severity:** Low. **Likelihood:** Common. **Effort:** Small.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-23, from `calummoore/h5-resumable-image-download-u7r9`,
  the review of pull request #127 by workstream A, finding 3.
- **Location:** `crates/ciproxy-agent/src/resumable_download/partial.rs`,
  function `Partial::open`.
- **Reason:** It costs disk reads and time, and the download still
  finishes.

- **Background:**
  - The agent on a Mac keeps the bytes of a broken macOS image download in
    `images/NAME.tar.partial`. A sidecar names the image ID, the SHA-256,
    and the byte size.
  - Each new `FetchImage` command with a matching sidecar reads and hashes
    the whole partial file before it sends `Range`.
  - A download URL expires after 2 hours. On a slow line, each expiry ends
    the command with a `403`, and the next command hashes the file again.
- **Problem:** A slow line hashes up to 60 GB once for each URL.
- **Impact if no action:** About 20 s of disk reads for each 60 GB, at
  each new command.
- **Root question:** How does the agent continue a partial file without a
  second read of the same bytes?
- **Options:**
  - **A.** Keep the SHA-256 state of the last command in memory in
    `ResumableDownload`, with the sidecar and the file length. Reuse it
    when both still match. Hash again after an agent restart. Effort:
    Small.
- **Left out:** A saved SHA-256 state on disk: `sha2` gives no stable
  form of its state.
- **Wider fix:** None.
- **Previous examples:** None found.
- **Related:** `macos-download-server-error-retry.md` makes the case less
  common.
- **Recommendation:** A.
- **Action:** fix after merge, apply A, because the cost is time only.
