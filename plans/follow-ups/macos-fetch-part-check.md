# A bad macOS input download keeps its final name

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-23, from `calummoore/h4-publish-and-xcode-fetch-gomg`,
  review round 2, a problem outside the scope.
- **Location:** `image/macos/fetch.sh`, the `fetch` helper.
- **Reason:** The change moves every download of the build, so it needs a
  build run on a Mac to check. The publish branch fixed only the Xcode
  block, which now checks each new archive once, on its `.part` file,
  before the rename.

- **Background:** `fetch.sh` downloads the restore image, the runner image
  source, the runner, and Packer to a `.part` file, renames it, then checks
  its SHA-256.
- **Problem:** A corrupt download keeps its final name. Each later run skips
  the download and fails the same check, until a person deletes the file.
- **Impact if no action:** A broken download of the large restore image
  stops each rebuild until a person finds and deletes the file.
- **Root question:** How does a rerun recover from a corrupt download?
- **Options:**
  - **A.** Move every input onto one helper: download to a `.part` file,
    check its SHA-256, then rename it, as the Xcode block does. The Xcode
    sources can then use the same helper. Effort: Small.
  - **B.** Do A, and delete a `.part` file that fails the check, so that the
    next run starts the download from zero. Effort: Small.
- **Left out:** A check before each run of every final file already happens.
  It does not recover.
- **Wider fix:** A. One helper then holds the rule for every input.
- **Previous examples:** The R2 Xcode download of `fetch.sh`.
- **Recommendation:** A. A resumed `.part` file can still finish right on a
  later run.
- **Action:** fix after merge, apply A, because the change needs a build run
  on a Mac to check.
