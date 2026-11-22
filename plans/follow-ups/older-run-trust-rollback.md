# Older runs can revert later trust changes

- **Severity:** High. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-22, from the branch
  `ops/production-image-trust-2026-11-22`, the blocked wider fix of review
  finding 1.
- **Source:** Review.
- **Category:** Security.
- **Location:** `deploy/scripts/promotion_event.py:15-50`,
  `deploy/scripts/shared_routing.py:34-46`,
  `deploy/scripts/ci_roots.py:14-19`,
  `deploy/scripts/terraform_policy.py:60-86`,
  `.github/workflows/deploy-infrastructure.yml:16-28`,
  `.github/workflows/deploy-main-preview.yml:13-19`, and
  `.github/workflows/deploy-production.yml:15-26`.
- **Reason:** Each option that closes the class adds a CI rejection or
  reverses automatic trust updates, and only the user can approve either.

- **Terms:**
  - **Trust roots:** the Terraform roots `deploy/terraform/seed` and
    `deploy/terraform/bootstrap`, which hold the CI accounts, their
    federation providers, and the project IAM grants.
  - **Prelaunch trust account:** `ciproxy-trust-preview-apply`, which main
    preview runs on `main` use to apply the trust roots until the launch
    retirement.
  - **Production trust account:** `ciproxy-trust-prod-apply`, which
    production runs use to apply the trust roots after the launch.
  - **Rerun:** a GitHub rerun of all jobs, which starts a new attempt of an
    earlier run with the commit and the event of that run.
  - **Launch retirement:** the section "Retire prelaunch authority" of
    `deploy/launch.md`, which disables the prelaunch shared, trust, and
    central plan accounts.
- **Background:**
  - CI applies the trust roots from the commit of each run. The trust plan
    and apply jobs run first, at
    `.github/workflows/deploy-infrastructure.yml:16-28`. They check out the
    source commit of the run.
  - Before the launch, main preview runs on `main` apply the trust roots
    through the prelaunch trust account.
    `deploy/scripts/shared_routing.py:34-46` and
    `deploy/scripts/ci_roots.py:14-19` select them.
  - After the launch, production runs apply the trust roots through the
    production trust account. Their commit is
    `github.event.workflow_run.head_sha`, at
    `.github/workflows/deploy-production.yml:15-26`.
  - On 5 November 2026, the user chose that CI plans, checks, and promotes
    every Terraform root after each merge.
    `plans/follow-ups/production-deny-policy.md` records that choice.
  - `deploy/promotion.md:150-153` gives the rule in force for the trust
    roots. CI never deletes, replaces, or forgets a trust resource.
    In-place trust updates stay automatic.
  - `deploy/scripts/terraform_policy.py:60-86` enforces that rule in CI. It
    refuses a delete, a replacement, or a forget of a trust resource. It
    allows an in-place update and a create.
  - A rerun keeps the commit and the event of its first attempt.
    `deploy/scripts/promotion_event.py:15-50` accepts any push commit on
    `main` for main preview. For production, it accepts any successful main
    preview push run on `main`.
  - No check compares the commit of a run with the head of `main`. No check
    compares it with the commit of the last trust apply. A new attempt
    cannot reuse an old saved plan, but it plans again from the same
    commit.
  - The shared, data, and edge roots behave the same way. This item covers
    the trust roots, where a revert changes access.
  - The image trust move shows the class. The user approved it on
    22 November 2026. It moved the claim of both image providers from the
    primary branch `calummoore/ci-proxy-load-balancer` to `main`, in the
    local `image_workflow` of `deploy/terraform/bootstrap/main.tf`.
  - Before the launch retirement, a rerun of a `main` push from before the
    move applies the primary-branch claim again. Its image job then calls
    the workflow on the primary branch and can authenticate. So the run can
    pass.
  - The launch guide has a direct mitigation for that case. "Move the
    binding to main at launch" in `deploy/image-workflow-bootstrap.md`
    reruns only the preview push of the current reviewed `main` commit. It
    also says not to rerun a `main` push from before the move.
  - "Permit production" in `deploy/launch.md` repeats the image provider
    check after the last operator bootstrap and before the production
    switch. Keep both mitigations until a chosen option replaces them.
- **Problem:** A rerun of an older run plans and applies the trust roots of
  its own commit. That apply silently reverts each later in-place trust
  change, such as a provider condition or a custom role. It can also
  create again a trust resource that a later operator recovery removed,
  because the CI policy allows creates.
- **Impact if no action:** After the launch, one rerun of an older
  production run, or of an older main preview run that production then
  follows, can undo a reviewed trust change. The trust accounts hold
  effective owner power, so the undo can reopen access with no failed
  check.
- **Root question:** How does CI keep each applied trust change in force
  when someone reruns a run of an older commit?
- **Options:**
  - **A.** Keep only the direct mitigations of the launch guide. Add the
    same rule to the other rerun steps of the deploy guides: rerun only the
    push of the current `main` commit. It adds no rejection. A writer who
    reruns an older run still reverts trust, and no check detects it.
    Effort: Small.
  - **B.** Add a head guard. Before a trust plan, CI compares the commit of
    the run with the current head of `main`. It refuses a different commit.
    This adds a CI rejection. It also refuses a superseded run whose commit
    is newer than the last trust apply. It needs one GitHub read in the
    plan job, tests, and doc updates. Effort: Medium.
  - **C.** Add an ancestry guard. Each trust apply records its commit, for
    example as a bootstrap output. The next trust plan refuses a commit
    that does not descend from the recorded commit. This adds a CI
    rejection and a recorded value. It refuses only commits older than the
    last apply. It needs the history or a GitHub compare read in the plan
    job, tests, and doc updates. Effort: Medium.
  - **D.** Make the trust roots operator-only. CI stops applying seed and
    bootstrap. The operator applies each trust change from the current
    `main`. This removes the class. It reverses the choice of 5 November
    2026 for the trust roots, so each trust change waits for an operator.
    Effort: Medium.
- **Left out:**
  - Move the launch proof after the launch retirement: it fails the goal.
    It covers only the launch, and production keeps the class. On
    22 November 2026, the user also required the approved launch order
    unchanged.
  - Add a marker resource to each trust change, so that the CI delete
    refusal stops older plans: C is better on every point. C needs no
    author to remember a marker.
  - Apply the trust roots from the head of `main` in place of the run
    commit: it breaks a requirement that stays. Production must promote the
    exact successful preview commit, as "Permit production" in
    `deploy/launch.md` says.
- **Wider fix:** D removes the class, because CI no longer writes the trust
  roots. C is the detection form, with a smaller change of behaviour.
- **Previous examples:** The image trust move of 22 November 2026, which
  the background gives. The review that added this item did not search the
  history for other cases.
- **Related:**
  - `plans/follow-ups/prelaunch-image-boundary.md` covers images that a
    primary-branch writer published before the same move. It is a separate
    problem. Neither fix replaces the other.
  - `plans/follow-ups/production-launch.md` follows `deploy/launch.md`. The
    launch uses the direct mitigations that the background gives.
- **Recommendation:** C. It refuses exactly the runs that would undo a later
  trust apply, and it keeps automatic in-place updates. D best protects the
  trust roots, but it reverses the choice of 5 November 2026.
- **Action:** ask the user, because the recommended guard adds a new CI
  rejection, and D reverses the user's choice of 5 November 2026.
