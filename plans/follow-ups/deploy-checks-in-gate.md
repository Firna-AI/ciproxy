# Repository gate skips the deploy tests

- **Severity:** Medium. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-23, from the branch
  `calummoore/h3-macos-release-from-r2-qjch`, review round 2, check of
  finding 9.
- **Location:** `xtask/src/check/phases.rs`, the gate phases;
  `.github/workflows/ci.yml`, the CI job; `deploy/README.md`, the offline
  checks.
- **Reason:** A new gate phase is a new gate, and the always-on rules
  reserve that choice for the user.

- **Terms:**
  - **Complete gate:** `cargo xtask check`, the repository check that CI
    runs for each push and pull request.
  - **Offline deploy checks:** the commands under "Offline checks" in
    `deploy/README.md`: Terraform `fmt`, `validate`, and `test`, Helm lint
    and templates, `actionlint`, and
    `python3 -m unittest discover -s deploy/tests -p 'test_*.py'`.
- **Background:** CI runs only `cargo xtask check --keep-going`. No gate
  phase runs the offline deploy checks. They need Terraform 1.17.1, Helm
  4.3.1, and `actionlint`, which `deploy/scripts/install-helm.sh` and the
  deploy workflows already pin or install.
- **Problem:** A change to `deploy/scripts/`, the Helm chart, or the
  Terraform modules can break a deploy test, and the pull request stays
  green. The R2 images key tests in `deploy/tests/test_r2_images_secret.py`
  and `deploy/terraform/data/r2_images.tftest.hcl` run only by hand.
- **Impact if no action:** A deploy regression shows up only in the next
  real deploy.
- **Root question:** How does each pull request run the deploy checks
  that need no cloud credentials?
- **Options:**
  - **A.** Add a CI job that installs the pinned tools and runs the
    offline checks of `deploy/README.md`. Effort: Small.
  - **B.** Add a `deploy` phase to `cargo xtask check` that runs the same
    commands and fails with a clear message when a tool is missing.
    Effort: Medium.
- **Left out:** Running them only before a deploy: the failure then comes
  after the merge.
- **Wider fix:** B. Local and CI runs then check the same things.
- **Previous examples:** `plans/follow-ups/image-checks-in-gate.md`, the
  same gap for the image checks.
- **Recommendation:** B. One gate for local runs and CI.
- **Action:** ask the user, because it adds a gate phase.
