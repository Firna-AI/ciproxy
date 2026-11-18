# Verify private CI with different job IDs

- **Severity:** High. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-18, from `conductor/job-id-binding`.
- **Location:** `docs/protocol/cache.md` and
  `crates/ciproxy-cache-sidecar/README.md`.
- **Reason:** The owner will run the live fixture after the fix merges and
  the agent release is available.

- **Terms:**
  - **Worker job ID:** The UUID reported by the runner's trusted handoff.
  - **Scale-set job ID:** The UUID in the hosted assignment for that runner.
  - **Bound pair:** Both UUIDs of the same job on one registered JIT runner.
- **Background:** ciproxy prepares each private GitHub job before its steps.
  On 17 November 2026, the owner chose to carry the bound pair in trusted
  facts. Preparation and cleanup use the Worker ID. A signed runtime-token
  `job_id` may match either UUID of that pair. Every other token check stays.
  This change requires the agent bundle release 0.1.3.
- **Task:** Cal will rerun the private CI fixture with different Worker and
  scale-set job IDs after the fix merges and release 0.1.3 is available.
  Capture safe job, registered runner, run, repository, image, and version
  metadata. Verify that preparation names the Worker ID and facts retain
  the exact pair. Confirm that steps start without a manual Drain or Resume.
  Check machine-cache use, metadata authorization, and session cleanup.
  Check that a signed job ID outside that binding fails its job check.
- **Impact if no action:** Local regressions prove the paired-ID behavior,
  but a published release has no live confirmation with different IDs.
- **Needs:** The merged control-plane fix and release 0.1.3 on the machine.
  The owner needs access to the private fixture and its safe observations.
- **Done when:** One live job with different IDs completes preparation and
  its steps. Its trusted facts, metadata access, and cleanup use the same
  verified binding. The token check accepts its bound job ID and preserves
  all other signature and identity checks. Save no raw token or credential.

The previous live sample was run `44202592271` in `Firna-AI/ciproxy-spike`.
It used control plane `9b094e15` and agent `0.1.2`. Its signed `job_id`,
hosted scale-set job ID, and Worker ID were all
`6d88988c-67b0-5e52-87a7-a2fa0c927e5e`.
RS256 verification used GitHub's public JWKS at
`https://token.actions.githubusercontent.com/.well-known/jwks` and key ID
`38826b17-6a30-5f9b-b169-8beb8202f723`.
That equal-ID sample cannot distinguish the two namespaces.

On 18 November 2026, live run 5 used agent 0.1.3 and control plane
`754a5048`. In `firna-test-org/test-repo-private`, the container job and
the hold job of fixture run `44308584322` and the complete gate of run
`44310052849` started their steps with no manual Drain or Resume. In each
of these jobs, the Worker ID equaled the scale-set job ID. So a live job
with different IDs is still not proved.
