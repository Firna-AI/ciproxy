# Runtime token job identity needs proof

- **Severity:** High. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-17, from `conductor/job-id-binding`.
- **Location:** `crates/ciproxy-github/src/assignments.rs:62` and
  `crates/ciproxy-cache-sidecar/src/runtime_token/claims.rs:80`.
- **Reason:** The required verified token identity is unavailable in this
  workspace, so the full preparation fix must wait.

- **Source:** Implementation.
- **Category:** Docs or spec.
- **Terms:**
  - **Worker ID:** The UUID in GitHub's `AgentJobRequestMessage.JobId`.
  - **Scale-set job ID:** The UUID in a typed scale-set assignment message.
  - **Runtime token:** The original access token from `SystemVssConnection`.
- **Background:** ciproxy prepares a private GitHub Actions job before its
  first step. The agent requires the preparation ID to match its Worker ID.
  The cache also checks a signed top-level `job_id` against the trusted facts.
  The owner selected an exact registered-runner binding on 17 November 2026.
  The owner required token-format proof before selecting the expected token ID.
- **Problem:** The observed Worker IDs differ from the hosted Started IDs.
  Public GitHub sources establish the Results backend ID in `scp`.
  They do not define the separate top-level `job_id` that the sidecar checks.
- **Impact if no action:** The binding can be stored and used for metadata.
  Preparation and trusted cache facts cannot safely receive their final ID.
- **Root question:** Which verified source establishes the token's top-level
  job identity for the exact registered runner?
- **Options:**
  - **A.** Provide one verified current claim report from the same JIT job.
    Include the Worker ID and typed scale-set job IDs in that report.
    This needs access to the private runner fixture. Effort: Medium.
  - **B.** Provide an official token schema or issuer source that defines the
    top-level `job_id` and its link to the Worker ID.
    No such source was found in the public runner or toolkit. Effort: Medium.
- **Left out:** A decoded unsigned payload proves no trusted identity.
  The Results `scp` segment does not define a different claim.
  A job name cannot prove a runner binding.
  A synthetic test token cannot prove GitHub's token format.
- **Wider fix:** A. Keep a verified comparison of the independent IDs with
  the crate's token-format proof.
- **Previous examples:** The checked live-validation task in
  `plans/ciproxy-mvp.md` reports a token-to-message match on 5 November 2026.
  It gives neither a typed message nor the Worker ID, so it cannot resolve
  the mismatch of the current fixture.
- **Recommendation:** A. A current verified observation can distinguish the
  IDs without treating different token claims as equivalent.
- **Action:** ask the user, because the explicit proof prerequisite needs
  verified private-fixture evidence that this workspace cannot read.

## Required safe evidence

Supply only metadata from one job, never the token or its encoded payload:

- Runner version and image identity.
- The exact numeric GitHub runner registration ID.
- The Worker ID read from the trusted handoff before job code starts.
- Scale-set job UUIDs, each with its message type and registration ID.
- The token's top-level `job_id`, `run_id`, `runner_id`, and `repository_id`.
- Verification results for signature, algorithm, trusted key source, issuer,
  and active validity interval.
- Claim names and the Results backend job ID, if that claim is present.

The existing `crates/ciproxy-github/scripts/runtime_token_probe.py` verifies
its configured signing profile before printing permitted identity metadata.
Its guide is `crates/ciproxy-github/scripts/README.md`.
Do not assume that an OIDC discovery document defines an access-token profile.
Keep every existing signature and identity check.

GitHub's pinned runner sources pass Worker `JobId` into Results
`WorkflowJobRunBackendId`.
The official artifact client reads that backend identity from the runtime
token's `Actions.Results:<run-backend-id>:<job-backend-id>` scope.
This establishes an interoperability contract for that scope.
It does not establish the value or presence of a separate `job_id` claim.

After the evidence resolves the ID, finish the preparation and facts wiring.
Run the complete differing-ID regression, metadata tests, cleanup tests, and
the agent's exact-match tests.
Run each new or changed asynchronous test ten times.
Run the complete gate with `DATABASE_URL` set.
Complete the runtime-fix branch integration before opening a ready PR.
