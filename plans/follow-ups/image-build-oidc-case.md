# Build OIDC case repeats the permission assertion

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-22, from the branch
  `ops/production-image-trust-2026-11-22`, outside the scope of review
  finding 5 of round 2.
- **Source:** Review.
- **Category:** Test.
- **Location:** `deploy/tests/test_image_workflow.py`, the subtest with the
  scenario "source build without OIDC" in
  `test_only_reusable_image_jobs_on_main_satisfy_real_provider_conditions`,
  and `admits()` in `deploy/tests/image_security_support.py`.
- **Reason:** The case dates from commit `6e2a4811` on `main`, and this
  branch changes only its arguments.

- **Terms:**
  - **Image provider:** a Google Cloud workload identity provider that
    checks a GitHub token before it grants `ciproxy-image-lookup` or
    `ciproxy-image-publish`.
  - **`admits()`:** the test helper that evaluates the planned condition
    of an image provider for a set of token claims.
- **Background:**
  - The build job of `.github/workflows/deploy-control-plane-image.yml`
    builds the image with no cloud identity. It has only `contents: read`
    and no GitHub environment.
  - The test first asserts that the build job's permissions equal
    `{"contents": "read"}`. It also asserts that the job has no
    `environment` key.
  - Then, for each image provider, the subtest "source build without OIDC"
    calls `admits()` with `oidc=permissions.get("id-token") == "write"`.
    It asserts that the result is false.
  - `admits()` returns false at once when `oidc` is false. It reads no
    condition and no claim.
  - The permission assertion before the subtest fixes `oidc` to false. So
    the subtest passes whenever that assertion passes.
  - Commit `6e2a4811` of 8 November 2026 added the case, and `main` has it.
  - No Python case evaluates the environment clause of the image
    providers.
    `deploy/terraform/bootstrap/control-plane-images.tftest.hcl` checks
    only that the text `assertion.environment == 'main-preview'` is in
    each condition.
- **Problem:** The subtest cannot fail while the permission assertion
  before it passes. It repeats that assertion, and it evaluates no provider
  condition.
- **Impact if no action:** The case reads like a proof that the providers
  refuse the build job, but it proves nothing beyond the permission check.
  An edit that keeps the environment text but stops that clause from
  refusing a job passes every check.
- **Root question:** How do the image tests prove, with no repeated
  assertion, that the build job cannot get an image identity?
- **Options:**
  - **A.** Delete the subtest. The permission and environment assertions
    stay, and they hold the requirement. The `testing` skill allows this
    removal, because the permission assertion in the same test covers the
    same requirement at the same level. Effort: Small.
  - **B.** Call `admits()` with `oidc=True` for the claims of the build
    job, and keep the false assertion. The subtest then shows that the
    providers also refuse an image workflow job without the `main-preview`
    environment. Effort: Small.
- **Left out:**
  - Change nothing: it fails the goal, because the case keeps a proof that
    it does not give.
  - Only two approaches are real. The case either goes, or it evaluates a
    provider condition, and B is the form that evaluates one.
- **Wider fix:** None. One case has the defect, and only a test for the
  class would catch every assertion that cannot fail.
- **Previous examples:** None known. The review that found it read only the
  image tests.
- **Recommendation:** B. It keeps the scenario and gives it a real failure:
  a provider condition that admits an image workflow job without the
  `main-preview` environment.
- **Action:** fix after merge, apply B, because the case predates this
  branch and is Low test maintenance.
