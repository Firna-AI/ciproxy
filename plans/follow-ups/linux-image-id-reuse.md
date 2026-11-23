# Linux releases can reuse one image ID

- **Severity:** High. **Likelihood:** Common. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-23, from the branch `calummoore/swap-support-3dll`,
  review finding 1, the wider fix.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `image/pins.env:3`, `image/export.sh:40`,
  `crates/ciproxy-control-plane/scripts/publish-job-image.py`,
  `crates/ciproxy-control-plane/src/image_delivery.rs:20`, and
  `crates/ciproxy-agent/src/images.rs:47`.
- **Reason:** The branch fixed its own release with a new ID, and a refusal
  at publication adds a new rejection that needs the user's approval.

- **Terms:**
  - **Image ID:** the value in `image/pins.env` that names one release of
    the Linux machine image. Export writes it into the release manifest,
    and provisioning writes it into the guest.
  - **Publisher:** `crates/ciproxy-control-plane/scripts/publish-job-image.py`,
    which uploads a built Linux release to the image bucket.
- **Background:**
  - The control plane offers a release only when a machine does not report
    its image ID (`crates/ciproxy-control-plane/src/image_delivery.rs:20`).
    The agent also skips a download for an ID that it has
    (`crates/ciproxy-agent/src/images.rs:47`).
  - The publisher accepts any release folder and does not compare IDs. The
    macOS publisher already refuses a reused ID (`image/macos/publish.sh:80`).
  - Before the swap branch, three published releases, named after commits
    `4a950ae2`, `6ed25c36`, and `b751ef5f`, all carried
    `ubuntu24-20261101.360-runner-2.338.2-handoff-v1` with different
    contents. By the code, a machine keeps the first of them that it
    installed.
  - The swap branch changed the ID to
    `ubuntu24-20261101.360-runner-2.338.2-swap-v1`, and added a release
    step in `image/README.md` that asks for a new ID with each release.
- **Problem:** Nothing stops a release with new contents from reusing an
  earlier ID. Machines that hold the earlier release then never get the
  new contents.
- **Impact if no action:** A security fix or a feature in the image can
  silently fail to reach installed machines.
- **Root question:** How does each Linux release with new contents reach
  the machines that hold an earlier release?
- **Options:**
  - **A.** Keep the manual step in `image/README.md`. Effort: Small.
  - **B.** Require an explicit release revision when either Linux image
    builds, and use it in both builds. Effort: Medium.
  - **C.** Derive the ID from the release source and the pinned inputs.
    Cost: it misses a rebuild of the same commit, because packages without
    a pin resolve at build time. Effort: Medium.
  - **D.** Make the publisher refuse a release when an existing release in
    the bucket has the same ID and another archive SHA-256, as the macOS
    publisher does. Cost: a new refusal, list and read rights on the
    bucket, and a fake `gcloud` for its test. Effort: Medium.
  - **E.** Add the archive SHA-256 to the delivery identity, so the control
    plane offers a release again when the digest differs. Cost: a change
    to the agent report, its protocol type, and the image store layout.
    Effort: Large.
- **Left out:** A new storage prefix alone, and a forced download by hand,
  because both leave installed machines on the old release.
- **Wider fix:** D. It stops any reuse, whatever changed.
- **Previous examples:** Commit `703b9b19` bumped the ID by hand when the
  runner handoff changed.
- **Recommendation:** D, with A or B to supply the IDs. It stops the class
  at the one place that sees every release.
- **Action:** ask the user, because D adds a refusal at publication.
