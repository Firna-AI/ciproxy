# A repeated macOS archive entry passes the read

- **Severity:** High. **Likelihood:** Rare. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-23, from the branch
  `calummoore/h3-macos-release-from-r2-qjch`, review round 2, finding 10.
- **Location:** `crates/ciproxy-control-plane/src/image_release.rs`, the
  `Platform::MacosArm64` arm of `SignedReleases::current`.
- **Reason:** Refusing a repeated entry is a new rejection for the macOS
  release, and only the user adds a rejection.

- **Terms:**
  - **`downloads.json`:** the release file that maps an archive key to
    its file name, SHA-256, and byte size. The Linux key is
    `firecracker`, the macOS key is `macos`.
- **Background:** The Linux reader decodes `downloads.json` into a typed
  struct, so a second `firecracker` entry stops the start. The macOS
  reader reads the file into a map and takes the `macos` entry, so other
  keys stay unread. A map keeps the last of two entries with one key, and
  `serde_json::Value` also keeps the last of two `file`, `sha256`, or
  `size_bytes` fields in one entry. `image/macos/package.sh` writes the
  file from a Python dict, which cannot repeat a key.
- **Problem:** A hand-edited or corrupt `downloads.json` with two `macos`
  entries gives a silent choice of one archive and its own matching hash.
- **Impact if no action:** Macs can get the wrong archive. The SHA-256
  check passes, because the hash comes from the same chosen entry.
- **Root question:** How does the macOS read get one archive entry, with
  one value for each field?
- **Options:**
  - **A.** Change nothing, because the publisher cannot write a repeated
    key. Effort: Small.
  - **B.** Add one decoder that takes the archive key, decodes that entry
    straight into the typed `Artifact`, refuses a second entry under the
    key with a typed error, and skips other keys. Use it for both
    platforms. A repeated `macos` entry then turns macOS images off with a
    warning. Effort: Small.
  - **C.** Refuse a repeated JSON name at every level of each release
    metadata read, for both files and both platforms. Effort: Medium.
- **Left out:** Keeping the first entry: it still accepts ambiguous
  metadata. Checks in the publisher only: stored releases stay unchecked.
- **Wider fix:** C. It works whatever decode shape comes later.
- **Previous examples:** The Linux reader before the shared reader, from
  commit `703b9b19`, refused a second `firecracker` entry.
- **Recommendation:** B. It gives both platforms one rule for the one
  file format, for a Small change.
- **Action:** ask the user, because the fix adds a new rejection of a
  macOS release file.
