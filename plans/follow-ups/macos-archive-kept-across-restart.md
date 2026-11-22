# An agent restart deletes a checked macOS image archive

- **Severity:** Low. **Likelihood:** Rare. **Effort:** Small.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-23, from `calummoore/h5-resumable-image-download-u7r9`,
  the review of pull request #127 by workstream A, finding 4. It is outside
  the scope of that pull request.
- **Location:** `crates/ciproxy-agent/src/platform/macos/images.rs`,
  function `recover`.
- **Reason:** Only an agent restart during the extraction meets it, and it
  costs time only.

- **Background:**
  - The agent on a Mac downloads `images/NAME.tar.partial`. After the size
    and the SHA-256 match, it renames the file to `images/NAME.tar`, then
    extracts the archive into `images/NAME.pending/`.
  - At start, `recover` keeps a partial file with a valid sidecar. It
    removes each `images/NAME.tar` and each `.pending` part.
- **Problem:** An agent restart or a crash during the extraction deletes a
  complete, checked archive of 30 to 60 GB.
- **Impact if no action:** The next `FetchImage` downloads the whole image
  again.
- **Root question:** How does a checked archive survive an agent restart?
- **Options:**
  - **A.** Keep the sidecar until the publication ends. At start, keep a
    `NAME.tar` with its sidecar, and check its size and SHA-256 again
    before the extraction. Effort: Small.
- **Left out:** A trust of the archive at start with no check: a crash can
  leave a damaged file.
- **Wider fix:** None.
- **Previous examples:** `repeat-install-image-download.md`, the same loss
  of download progress on Linux.
- **Related:** None.
- **Recommendation:** A.
- **Action:** fix after merge, apply A, because only a restart during the
  extraction meets it.
