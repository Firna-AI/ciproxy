# Startup builds each image offer in two places

- **Severity:** Medium. **Effort:** Medium. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-23, from the branch
  `calummoore/h3-macos-release-from-r2-qjch`, review round 1, findings 6
  and 8.
- **Location:** `crates/ciproxy-control-plane/src/application.rs`,
  `Production::serve`; `crates/ciproxy-control-plane/src/macos_release.rs`,
  `download_signing` and `read_macos_release`;
  `crates/ciproxy-control-plane/src/image_signing.rs`, `PlatformSigning`;
  `crates/ciproxy-control-plane/src/composition.rs`, `Hosted::new`.
- **Reason:** The fix moves the startup and composition code that the
  per-platform image work had just settled, with `PlatformSigning` kept on
  request of the parent workspace.

- **Terms:**
  - **Image offer:** `ImageOffer` in `image_offers.rs`. The release facts
    and the signer that the runtime uses for each `FetchImage`.
- **Background:** At start, `Production::serve` reads the Linux release
  with the CDN key and the macOS release with a 5-minute R2 signer, and
  puts both downloads in `Config.images`. `Hosted::new` then decodes the
  CDN key again, parses `R2_IMAGES_KEY` again through `download_signing`,
  and pairs each download with a signer by platform in
  `PlatformSigning::offers`. A macOS release without its signer is left
  out with a warning.
- **Problem:** Two places read the same settings to build one offer. The
  in-process tests build offers themselves, and the executable test checks
  only that a refused macOS read leaves the start ready. No test shows that
  a read macOS release reaches the runtime with its 2-hour signer through
  normal startup.
- **Impact if no action:** A change to one path can lose the macOS offer
  at start, and only the warning in the log would show it.
- **Root question:** How does startup build each release and the signer of
  its downloads together, from one read of their settings, under test?
- **Options:**
  - **A.** Parse `R2_IMAGES_KEY` once and pass the parsed key to
    composition. Effort: Small.
  - **B.** Let the shared release reader return the complete offer: a
    download signer beside its read signer, the CDN key for Linux and a
    2-hour R2 signer from the same parsed key for macOS. Startup passes
    the offers to composition, and `Config.images`, `PlatformSigning`,
    and `download_signing` go. Add one public startup function and test it
    in process with local CDN and R2 stores and a fixed clock. Effort:
    Medium.
- **Left out:** Reading the macOS release inside composition: it makes
  `Hosted::new` asynchronous and changes the start order. A lazy read in
  the runtime: it moves the read after readiness, against the start-time
  read with no retry that the user chose on 23 November 2026.
- **Wider fix:** B. The reader owns the link between a release and its
  signer for every platform.
- **Previous examples:** Commit `99659035`, which added the per-release
  signer.
- **Recommendation:** B. One owner, and one test at the startup boundary.
- **Action:** fix after merge, apply B, because it restructures startup
  code that the per-platform image change and its parent just settled.
