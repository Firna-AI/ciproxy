# The macOS manifest writer takes two test flags

- **Severity:** Low. **Likelihood:** Rare. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-23, from `calummoore/h4-publish-and-xcode-fetch-gomg`,
  review round 3, a problem outside the scope.
- **Location:** `image/macos/write-manifest.py`, the `--variant` and
  `--test-image` arguments; `image/macos/build.sh`, `phase_test_image`.
- **Reason:** The publish branch did not change the manifest writer.

- **Terms:**
  - **Variant:** the `variant` field of `manifest.json`: `full`,
    `standard`, or `test`.
- **Background:** `write-manifest.py` takes `--variant` and `--test-image`
  as separate arguments and writes both fields. `publish.sh` refuses a
  manifest whose `variant` and `test_image` disagree, because only the
  `test` variant is a test image. `phase_test_image` passes both
  `--test-image` and `--variant test`, so today's build writes a pair that
  agrees.
- **Problem:** A call with `--variant full --test-image`, or `--variant test`
  alone, writes a manifest that `publish.sh` refuses only at publish time.
- **Impact if no action:** A wrong call wastes a package run, because the
  error shows only when the operator publishes.
- **Root question:** How does the manifest writer write only pairs that
  agree?
- **Options:**
  - **A.** Derive `test_image` from the variant and drop `--test-image`.
    Removing the flag needs the user's approval, because it is on `main`.
    Effort: Small.
  - **B.** Keep both arguments, and refuse a pair that disagrees in
    `write-manifest.py`. Effort: Small.
- **Left out:** No change leaves the error until the publish.
- **Wider fix:** A. One field then holds the fact.
- **Previous examples:** None found.
- **Recommendation:** B. It needs no approval and catches the error at
  build time.
- **Action:** ask the user, because A removes a flag that is on `main`, and
  B adds a rejection to the manifest writer.
