# A wrong manifest path can exhaust pod memory

- **Severity:** High. **Likelihood:** Rare. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-23, from the branch
  `calummoore/h3-macos-release-from-r2-qjch`, review round 1, finding 5.
- **Location:** `crates/ciproxy-control-plane/src/image_release.rs`,
  `SignedReleases::read`; `deploy/helm/ciproxy/values.yaml`, the 1 GiB
  memory limit.
- **Reason:** A byte limit adds a new rejection of release metadata, and
  the always-on rules reserve that choice for the user.

- **Terms:**
  - **Release metadata:** `manifest.json` and `downloads.json` of one
    image release. Valid files are a few kilobytes.
- **Background:** At start, the control plane reads the Linux and the
  macOS release metadata with `Response::json`, which buffers the whole
  body. `MACOS_IMAGE_MANIFEST_PATH` and `JOB_IMAGE_MANIFEST_PATH` are
  free paths. The macOS release folder also holds `macos.tar`, which is
  30 to 60 GB. The macOS reads have a 30-second request limit, and the
  Linux reads 90 seconds. The rolling update keeps the old pod until a
  new pod is ready.
- **Problem:** A path that names a large object makes the reader buffer
  it until the pod passes its memory limit and is killed. The log then
  shows only `OOMKilled`, with no line that names the read. A failed
  macOS read should only turn macOS images off.
- **Impact if no action:** The deploy fails after the 10-minute Helm
  wait. A start with no old pod blocks Linux and Boat until the setting
  is fixed.
- **Root question:** How does every response from a release store,
  however large, end in a named outcome and never stop the process?
- **Options:**
  - **A.** Limit each macOS metadata read to 1 MiB as it streams in, and
    give the macOS-off warning when it is passed. Effort: Small.
  - **B.** Apply one 1 MiB limit in `SignedReleases::read` to the Linux
    and the macOS reads, with the `chunk()` loop of
    `crates/ciproxy-overflow/src/boat/http.rs`. An oversized Linux read
    then stops the start with a named error. Effort: Small.
  - **C.** B, plus a bounded body helper for the control-plane crate and
    a Clippy `disallowed-methods` rule for `Response::json`, `bytes`, and
    `text`. Adds a lint. Effort: Medium.
  - **D.** Let deploy resolve the release before Helm runs, with its own
    byte limit, and pass the facts to the pod. Effort: Large.
- **Left out:** A timeout alone and a header check alone: neither bounds
  the body. A higher memory limit: a 60 GB object is larger than any
  limit. A path check: it stops only the likeliest wrong path.
- **Wider fix:** C stops a new unbounded body read anywhere in the crate.
- **Previous examples:** None found for release metadata.
- **Recommendation:** B. It costs the same as A and also turns an
  oversized Linux read into a named error.
- **Action:** ask the user, because a byte limit adds a new rejection of
  release metadata.
