# The macOS publish step restates deploy settings

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Wider fix.
- **Added:** 2026-11-23, from `calummoore/h4-publish-and-xcode-fetch-gomg`,
  review round 2, the wider fix of finding 10.
- **Location:** `image/macos/README.md`, the paragraph after the steps of
  "Publish".
- **Reason:** The home of the macOS deploy settings comes with the branch
  `calummoore/h3-macos-release-from-r2-qjch`. A link to it fails the Markdown
  check until that branch merges.

- **Background:** The publish README tells the operator to set
  `MACOS_IMAGE_MANIFEST_PATH` and names `MACOS_IMAGE_URL_PREFIX` and
  `R2_IMAGES_KEY`. The deploy docs of H3 define these settings in
  `deploy/runtime.md` and `deploy/setup.md`.
- **Problem:** Two docs state the same settings, so they can drift.
- **Impact if no action:** A later change of a deploy setting can leave the
  image README wrong.
- **Root question:** Where does the operator read the deploy settings of a
  macOS release?
- **Options:**
  - **A.** After H3 merges, replace the setting names in the image README
    with a link to the macOS section of `deploy/runtime.md`. Keep the step
    "set the printed path, then deploy". The Markdown check then catches a
    broken link. Effort: Small.
- **Left out:** No change keeps two copies of one fact.
- **Wider fix:** A.
- **Previous examples:** None found.
- **Recommendation:** A.
- **Action:** fix after merge, apply A, because the link target exists only
  after H3 merges.
