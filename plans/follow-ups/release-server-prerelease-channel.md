# Deploy the release channel and installer handoff

- **Severity:** High. **Effort:** Small.
  **Confidence:** High. **Harder later:** Yes.
- **Kind:** Post-merge task.
- **Added:** 2026-11-08, from plan item 103, option B, which the user asked
  to build in parallel.
- **Location:** `deploy/scripts/mirror-release-image.sh`,
  `deploy/scripts/deploy_support.py`, and `deploy/release-server.md`.
- **Reason:** The code is complete, but the operator must mirror its image,
  set the digest, and verify the deployed change after the merge.

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
- **Task:** The orchestrator completes one operator rollout after merge:
  1. Mirror the pinned server image with
     `deploy/scripts/mirror-release-image.sh`, as `deploy/setup.md` describes.
  2. Record the returned digest. Set `RELEASE_SERVER_IMAGE_DIGEST` in each
     selected GitHub environment, then redeploy with fresh plans.
  3. Use `deploy/release-server.md` to check the live image and channel.
     Prove that preview selects a matching pre-release while production
     stays on full releases. Run the install command on a test machine.
     Confirm that the agent receives the install arguments and registers.
     For preview, use an agent release that accepts `--agent-url`, then
     copy the dashboard command. It supplies the issuing control plane's
     endpoint and saves it for the installed service.
     Keep all token values out of logs.
- **Impact if no action:** The old image keeps serving full releases only.
  Its installer downloads the agent but does not start `install --token`.
  Publishing a test agent as a full release can reach every customer.
- **Needs:** The merged ciproxy change, operator image and GitHub access,
  access to the deployed environments, and a test machine for registration.
- **Done when:** The selected environments run the mirrored revision.
  Main preview follows `prerelease`, and production follows `stable`.
  A newer matching pre-release cannot become production's latest release.
  The one-line install command registers the test machine.
