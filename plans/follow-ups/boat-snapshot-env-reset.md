# Boat snapshot steps reset the environment to preview

- **Severity:** High. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-22, from the branch
  `ops/production-image-trust-2026-11-22`, outside the scope of review
  finding 4 of round 2.
- **Source:** Review.
- **Category:** Docs or spec.
- **Location:** `deploy/runtime.md:137-156`.
- **Reason:** The steps are in `deploy/runtime.md`, which this branch does
  not change, and the later production runtime task runs them.

- **Terms:**
  - **Boat share code:** the code of the shared Boat snapshot of a job
    image, which the operator keeps out of shell arguments, files, and
    logs.
  - **Snapshot secret:** the Secret Manager secret
    `ciproxy-<env>-boat-shared-snapshot`, one for each environment, whose
    `latest` version deploy reads.
  - **Primary branch:** `calummoore/ci-proxy-load-balancer`, an unprotected
    branch whose pushes also deploy main preview.
- **Background:**
  - `deploy/runtime.md:137-139` says to run one block after the main
    preview data apply. It then says: "Repeat them for
    `DEPLOY_ENV=production` after its data apply succeeds."
  - The block at `deploy/runtime.md:141-156` starts with
    `export DEPLOY_ENV=main-preview`. It then reads the snapshot secret of
    `$DEPLOY_ENV`. Last, it adds the share code from a hidden prompt as a
    new version of that secret.
  - Commit `698cd176` of 7 November 2026 added the block.
  - Each environment deploy identity can read its own snapshot secret, as
    `deploy/runtime.md:133-135` says. Deploy reads the `latest` version,
    at `deploy/runtime.md:164`.
  - Main preview accepts pushes from the primary branch, as
    `deploy/identities.md:184` says. Its workload job runs the deploy
    scripts of the pushed commit with the preview deploy identity.
  - "Shared command blocks" in `deploy/README.md` says that a shared block
    takes its inputs from the step that runs it and never overwrites them.
    The Boat snapshot block overwrites `DEPLOY_ENV`.
- **Problem:** An operator who sets `DEPLOY_ENV=production` and then runs
  the block gets `main-preview` from its first line. The production share
  code then becomes the latest version of the preview snapshot secret.
  Every command succeeds, so nothing shows the mistake.
- **Impact if no action:** Production Boat stays off, and main preview
  deploys the production share code. Code on the unprotected primary
  branch can read that code through the preview deploy identity.
- **Root question:** How does each run of the Boat snapshot steps store the
  share code in the secret of the intended environment?
- **Options:**
  - **A.** Remove `export DEPLOY_ENV=main-preview` from the block. Before
    the block, tell the operator to set `DEPLOY_ENV` first: `main-preview`
    after its data apply, then `production` after its data apply. This
    follows the shared-block rule. Effort: Small.
  - **B.** Give each environment a complete block with its own `export`
    line. This copies the prompt code a second time. Effort: Small.
  - **C.** Move the steps into an operator script that takes the
    environment as a required argument. It adds code and a test for steps
    that run twice. Effort: Medium.
- **Left out:** A check in the block that stops when `DEPLOY_ENV` is
  unset: alone, it fails the goal, because the block still overwrites the
  value. After A, an unset value already fails at the first command,
  because it names no existing secret.
- **Wider fix:** None. "Shared command blocks" in `deploy/README.md` is
  already in force for this class.
- **Previous examples:** The image provider read of
  `deploy/image-workflow-bootstrap.md` had the same shape until
  22 November 2026. Its activation block set its own claim, and two later
  steps reused the block with the claim on `main`.
- **Related:** `plans/follow-ups/production-launch.md` lists the task that
  sets the Boat snapshot secret of production with these steps. Apply this
  fix before that task.
- **Recommendation:** A. It removes the overwrite with a small doc change,
  and it follows the shared-block rule.
- **Action:** fix after merge, apply A, because `deploy/runtime.md` is
  outside this branch, and the later runtime task runs these steps.
