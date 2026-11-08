# The release server has no pre-release channel

- **Severity:** High. **Effort:** Medium.
  **Confidence:** High. **Harder later:** Yes.
- **Kind:** Fix after merge.
- **Added:** 2026-11-08, from plan item 103, option B, which the user asked
  to build in parallel.
- **Source:** Implementation, the Milestone 13 live check.
- **Category:** Product bug.
- **Location:** `futex-ai/cli-release`,
  `crates/cli-release-server/src/github/releases.rs:21`, and
  `deploy/scripts/mirror-release-image.sh` with the release values of
  `deploy/helm/`.
- **Reason:** Item 103 = A made the first agent release a full release.
  Before the first customer update, a test release must reach the preview
  only.

- **Terms:**
  - **Full release:** a GitHub release that is not marked as a pre-release.
  - **Agent update:** the agent on each machine updates itself from the
    release server.
- **Background:** The installer and the agent update ask the release server
  for the latest release. The server asks GitHub for its latest release,
  which is the newest full release. The preview and production read the
  same repository, `Firna-AI/ciproxy`.
- **Problem:** A release that is published to test it on the preview also
  reaches every customer machine.
- **Impact if no action:** After launch, an untested agent can reach every
  customer at once.
- **Root question:** How does the preview get test releases that production
  does not?
- **Options:**
  - **A.** Add a channel setting to the release server. The preview follows
    the newest release, pre-releases included. Production follows full
    releases only. Mirror the new server image, and set the channel in the
    release values of each environment. Effort: Medium.
- **Left out:** A separate repository for preview releases, because the
  same tags would then live in two places.
- **Wider fix:** A.
- **Recommendation:** A.
- **Action:** fix after merge, apply A, because the user asked for it in
  parallel with the live check.
