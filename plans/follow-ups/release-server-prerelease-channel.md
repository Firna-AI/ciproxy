# Production release channel is not deployed

- **Severity:** High. **Effort:** Small.
  **Confidence:** High. **Harder later:** Yes.
- **Kind:** Post-merge task.
- **Added:** 2026-11-08, from plan item 103, option B, which the user asked
  to build in parallel.
- **Location:** `deploy/scripts/mirror-release-image.sh`,
  `deploy/scripts/deploy_support.py`, and `deploy/release-server.md`.
- **Reason:** The main preview part passed on 17 November 2026, and
  production exists only after the launch merge.

- **Terms:**
  - **Full release:** a GitHub release that is not marked as a pre-release.
  - **Release channel:** the server setting that selects the release for
    installs and agent updates.
- **Background:** The installer and agent updates ask the release server
  for the latest release. Preview and production read `Firna-AI/ciproxy`.
  The first agent release became a full release to unblock the live check.
  The user chose a separate preview channel on 8 November 2026.
  The user also confirmed the merged-source approach on that date, without
  publishing an upstream release.

  The upstream Clippy fix, release channel, and installer handoff changes
  merged into `futex-ai/cli-release` on 8 November 2026.
  The merged revision is `53a1dd3426226a9d25defb4a899d3c9a430a3941`.
  The ciproxy code pins that revision and its exact chart files.
  It renders `prerelease` for `main-preview` and `stable` for `production`.
  The archive contract is unchanged.
  The new installer runs the installed binary with its arguments through
  `exec`, which enables the one-line `install --token` command.

  On 17 November 2026 the main preview ran the mirrored revision with the
  `prerelease` channel. Both preview release endpoints served the
  pre-release `ciproxy-agent-v0.1.2`. The one-line install command of the
  preview dashboard installed agent 0.1.2 and registered the test machine
  in one call. Every agent release so far after the first is a
  pre-release.
- **Task:** The orchestrator completes the production rollout after the
  launch merge, during `production-launch.md`:
  1. Set `RELEASE_SERVER_IMAGE_DIGEST` of the production environment to
     the mirrored digest. Mirror it again with
     `deploy/scripts/mirror-release-image.sh` only when the pin changed, as
     `deploy/setup.md` describes.
  2. Publish the first full agent release, which is not a pre-release,
     through `.github/workflows/agent-release.yml`.
  3. Use `deploy/release-server.md` to check the live image and channel of
     production. Prove that production serves the full release and that a
     newer matching pre-release does not become its latest release.
  4. Run the one-line install command of the production dashboard on a
     test machine. Confirm that the agent receives the install arguments
     and registers. Keep all token values out of logs.
- **Impact if no action:** The old image keeps serving full releases only.
  Its installer downloads the agent but does not start `install --token`.
  Publishing a test agent as a full release can reach every customer.
- **Needs:** The launch merge, operator GitHub access, access to the
  production environment, and a test machine for registration.
- **Done when:** Production runs the mirrored revision and follows
  `stable`. A newer matching pre-release cannot become production's latest
  release. The one-line install command registers a test machine on
  production.
- **Related:** `production-launch.md` holds this rollout as one of its
  steps.
