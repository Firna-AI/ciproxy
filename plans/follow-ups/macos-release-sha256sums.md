# A published macOS release fails its own SHA256SUMS

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-23, from `calummoore/h4-publish-and-xcode-fetch-gomg`,
  review round 2, a problem outside the scope.
- **Location:** `image/macos/package.sh`, the `SHA256SUMS` writer.
- **Reason:** The file list of `SHA256SUMS` belongs to the packaging of PR
  #113. The bucket layout of the macOS image hosting plan fixes which files
  a release holds.

- **Background:** `package.sh` writes `SHA256SUMS` for the artifact folder.
  `publish.sh` checks it there, then uploads it with the release.
- **Problem:** `SHA256SUMS` also names `bundle/disk.img`, `bundle/aux.img`,
  `bundle/hardware-model.bin`, and `macos-content.sha256`. A release folder
  in R2 holds none of them, so `shasum -c SHA256SUMS` fails on a downloaded
  release.
- **Impact if no action:** A person who checks a downloaded release gets
  "No such file" errors and cannot tell them from a real failure. The agent
  does not read `SHA256SUMS`, so no Mac is affected.
- **Root question:** What does the `SHA256SUMS` of a release cover?
- **Options:**
  - **A.** Write two files: `SHA256SUMS` for the files of the release, and
    a build-side list for the bundle files. Effort: Small.
  - **B.** Document `shasum -c --ignore-missing` for a downloaded release.
    Effort: Small.
- **Left out:** Upload of the bundle files breaks the bucket layout and
  doubles the upload.
- **Wider fix:** A.
- **Previous examples:** None found.
- **Recommendation:** A. Each list then passes where it lives.
- **Action:** fix after merge, apply A, because the packaging belongs to the
  open macOS image build branch.
