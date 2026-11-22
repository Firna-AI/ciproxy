# A server error ends the macOS image download command

- **Severity:** Low. **Likelihood:** Rare. **Effort:** Small.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Ask the user.
- **Added:** 2026-11-23, from `calummoore/h5-resumable-image-download-u7r9`,
  the review of pull request #127 by workstream A, finding 2.
- **Location:** `crates/ciproxy-agent/src/resumable_download/answer.rs`,
  function `read_answer`.
- **Reason:** The requirement says that an HTTP error status ends the
  command. A change of that rule is a decision.

- **Background:**
  - The agent on a Mac continues a broken download of the macOS image with
    HTTP `Range` requests. A broken stream retries 3 times, 5 s apart,
    inside one `FetchImage` command.
  - Each HTTP error status, `4xx` and `5xx`, ends the command and keeps the
    bytes. The next `FetchImage` brings a fresh URL, and it hashes the
    bytes on disk again before it continues.
- **Problem:** A short `503` or `429` from R2 or a CDN ends the command, as
  a `403` for an expired URL does. The next command hashes up to 60 GB
  again.
- **Impact if no action:** A short host error costs one command round and
  one hash of the partial file. The download still finishes.
- **Root question:** Which HTTP answers retry inside one command?
- **Options:**
  - **A.** Keep the rule: each HTTP error status ends the command.
    Effort: None.
  - **B.** Retry `5xx` and `429` inside the command, with the rule for a
    broken stream. Other `4xx` statuses still end the command. Update the
    requirement in `docs/protocol/agent-images.md`. Effort: Small.
- **Left out:** A retry of each `4xx`: a `403` for an expired URL needs
  the fresh URL of the next command.
- **Wider fix:** None.
- **Previous examples:** None found.
- **Related:** `macos-download-hash-reuse.md` lowers the cost of option A.
- **Recommendation:** B. A short host error then costs 5 s, not a new
  command and a hash of the file.
- **Action:** ask the user, because the change alters a requirement.
