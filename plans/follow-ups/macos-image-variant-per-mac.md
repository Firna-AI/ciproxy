# Offer the full macOS image to Macs that fit

- **Severity:** Medium. **Effort:** Large. **Confidence:** Medium.
  **Harder later:** Yes.
- **Kind:** Needs decision.
- **Added:** 2026-11-23, from `calummoore/h4-publish-and-xcode-fetch-gomg`,
  option C of the decision on macOS image variants. Approved as a
  follow-up, 23 November 2026.
- **Location:** `crates/ciproxy-control-plane/src/image_release.rs`, the
  release key and the offer; `image/macos/write-manifest.py`, the
  `variant` field; `ts/app`, the dashboard; `ts/site` and
  `docs/protocol/agent-images-macos.md`, the docs.
- **Reason:** Only the standard image fits every Mac today, so the
  environments offer it alone until this work picks a variant per Mac.

- **Terms:**
  - **Variant:** the software set of the macOS image, in the `variant`
    field of `manifest.json`. `full` keeps the upstream simulator
    runtimes, such as Android. `standard` leaves them out.
  - **Release key:** the key under which the control plane holds one
    release. Today it is the platform, so each environment offers one
    macOS release.
- **Background:**
  - Decision, 23 November 2026, by the parent workspace H under Cal's
    authority. The options were standard everywhere, full everywhere, or
    a variant for each Mac or label. The choice was standard everywhere,
    because only the standard image fits every Mac today.
  - Measured sizes: the full image is 73.9 GB to download and 121.7 GB on
    disk. The standard image is 47.6 GB to download and 78.9 GB on disk.
  - The control plane offers one release for each platform, and each
    environment pins it with `MACOS_IMAGE_MANIFEST_PATH`.
- **Problem:** A customer who needs Android or the other simulator runtimes
  cannot get the full image, also on a Mac with room for it.
- **Impact if no action:** macOS jobs that need those runtimes fail or
  install them in each job.
- **Root question:** How does each Mac get the variant that its jobs need
  and its disk can hold?
- **Options:**
  - **A.** Pick the variant by the free disk of each Mac: full when it fits
    with room for jobs, else standard. The release key becomes platform
    plus variant, the control plane offers per variant, and each
    environment pins one release of each variant. Effort: Large.
  - **B.** Pick the variant by a label setting in the dashboard, with the
    same release key and offer change as A. The dashboard and the docs
    gain the setting. Effort: Large.
  - **C.** Do B, with A as the default when the label sets no variant.
    Effort: Large.
- **Left out:** Full everywhere: the full image does not fit every Mac
  today.
- **Wider fix:** None. The change is the feature itself.
- **Previous examples:** None found.
- **Recommendation:** C. A label states what the jobs need, and the disk
  rule gives a safe default.
- **Action:** ask the user, because the choice adds a product setting and
  changes what each Mac downloads.
