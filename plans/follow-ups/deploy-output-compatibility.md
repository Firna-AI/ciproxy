# A new deployment output can block the preview deploy

- **Severity:** High. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-23, from the branch
  `calummoore/h3-macos-release-from-r2-qjch`, review round 2, finding 9.
- **Location:** `deploy/scripts/deploy.py` and
  `deploy/scripts/optional_secret.py`, each read of the Terraform
  `deployment` output; `deploy/tests/fake_promotion_tools.py`, the fake
  `terraform output deployment`; `deploy/promotion.md`.
- **Reason:** The wider fix is a test for a class of defect, which needs
  the user's approval.
- **Approval:** Approved 23 November 2026 by the parent workspace H, under
  Cal's authority; fix after merge.

- **Terms:**
  - **Launch handoff:** the point after which production, not preview,
    applies the central Terraform roots, such as `data`. After it,
    preview only reads them (`deploy/scripts/ci_roots.py`,
    `deploy/scripts/shared_routing.py`).
  - **`deployment` output:** the Terraform output of the `data` root that
    the workload deploy reads, such as `r2_images_secret`.
- **Background:** Production applies preview data only after a
  successful preview run of the same commit (`deploy/promotion.md`). So
  after the launch handoff, a commit that reads a new field of the
  `deployment` output runs its preview workload against a data state
  without that field. The R2 images key now accepts a state without its
  field and applies an empty Kubernetes Secret. The Boat snapshot parent
  stays strict, because an empty Boat Secret stops Boat cleanup. The fake
  `terraform output deployment` always holds every field.
- **Problem:** Any later direct read of a new `deployment` field raises
  `KeyError` in the preview deploy after the launch. Production then
  cannot run the apply that would add the field. No test fails, because
  the fake output holds every field.
- **Impact if no action:** One such change can block every deploy until
  an operator applies preview data by hand.
- **Root question:** After the launch handoff, how does a commit deploy
  when it reads an output that the preview data state lacks?
- **Options:**
  - **A.** Merge each change with a new output before the launch handoff.
    Record the order in `plans/follow-ups/production-launch.md`. Effort:
    Small.
  - **B.** Catch a missing `deployment` field in `deploy.py` and print its
    name and a recovery link. Add the operator steps to `deploy/promotion.md`:
    `python3 deploy/scripts/terraform.py plan production data --target main-preview`,
    then `apply`, then rerun all jobs. Effort: Small.
  - **C.** Make the fake `terraform output deployment` return only the
    fields that both data states hold, from one committed fixture, so a
    new direct read of a newer field fails the deploy tests. Add a rule to
    `deploy/promotion.md` that a new required output needs B. Effort:
    Medium.
- **Left out:** A parent made by hand: it does not add the missing output
  field, and it conflicts with the apply. Production applying preview
  data before the preview run: it breaks the rule that production only
  promotes a commit that preview proved.
- **Wider fix:** C. It catches a direct read of any newer output.
- **Previous examples:** The Boat snapshot change, commit `698cd176`,
  added a direct read of a new output.
- **Recommendation:** C with B. C finds the defect before the merge, and
  B gives the recovery when it still happens.
- **Action:** fix after merge, apply C with B, the recommended option,
  which the approval above allows.
