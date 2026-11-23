# Two environments split the jobs of one organization

- **Severity:** Medium. **Likelihood:** Common. **Effort:** Small.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-23, from pull request #142
  (`conductor/ciproxy-runner-group-per-environment`), the check of review
  finding 5.
- **Location:** `crates/ciproxy-agent/scripts/own_smoke.py:105`,
  `crates/ciproxy-agent/scripts/own_scenarios.py:98`,
  `crates/ciproxy-github/scale-sets.md`, section "Runner groups".
- **Reason:** The change that found it only separates the runner groups
  of two environments. Which environment runs which job is a separate
  choice of the operator.

- **Terms:**
  - **Environment:** one deployed control plane with its own GitHub App,
    such as main preview or production. `deploy/README.md`, section
    "Environments", lists them.
  - **Managed runner group:** the private GitHub runner group that one
    environment keeps in each organization. It holds one scale set for
    each label of the environment's workspace.
  - **Smoke test:** the operator scripts under `crates/ciproxy-agent/scripts/`
    that dispatch workflow fixtures and check where their jobs run.
- **Background:**
  - Since pull request #142, two environments keep separate groups in one
    organization. Each group can hold a scale set with the same label
    name, such as `ciproxy`, and each can allow the same repository.
  - GitHub's runner scale-set guide, in its high-availability section,
    says: "If both runner scale sets are online, jobs assigned to them
    will be distributed arbitrarily (assignment race)." and "You cannot
    configure the job assignment algorithm."
  - GitHub's guide "Choosing the runner for a job" documents `runs-on`
    with `group` and `labels` for self-hosted runners. GitHub's scale-set
    pages do not show `group`, and no live probe has checked it for
    ciproxy's scale sets.
  - The production App is private, so only `Firna-AI` can install it. The
    main preview App already serves `Firna-AI`. A new organization allows
    all private repositories by default.
  - The smoke test dispatches its jobs to the label `ciproxy`
    (`runner_label="ciproxy"` in `own_smoke.py`). Its disallowed scenario
    in `own_scenarios.py` fails when any runner takes the job of a
    repository that preview does not allow.
- **Problem:** When production also serves `Firna-AI` with the default
  repository choice, GitHub can give a preview smoke job, or any job of a
  shared label, to production's scale set. The disallowed scenario then
  fails, and other jobs run in the other environment.
- **Impact if no action:** Smoke tests of main preview fail or pass on
  the wrong environment, and the jobs of `Firna-AI` run in whichever
  environment GitHub picks.
- **Root question:** How does each job of an organization that two
  environments serve run in the environment that it is meant for?
- **Options:**
  - **A.** Give production's `Firna-AI` registration a repository choice
    that leaves out the repositories of the preview smoke tests, in
    Settings under Organizations. Needs dashboard access. Effort: Small.
  - **B.** Give the preview workspace a label that production does not
    have, and dispatch the smoke jobs to it with `runner_label`. Effort:
    Small.
  - **C.** Let the smoke workflow name preview's runner group with
    `runs-on.group`, after a live probe shows that GitHub applies `group`
    to scale sets. Needs a live probe. Effort: Medium.
  - **D.** Keep production out of `Firna-AI`. Effort: Small.
- **Left out:** A change to ciproxy's routing. Each environment cannot see
  the scale sets of the other, and GitHub chooses among eligible scale
  sets.
- **Wider fix:** None. The choice is per organization and per operator.
- **Previous examples:** None found.
- **Related:** `plans/follow-ups/production-launch.md` is the step where
  production first serves `Firna-AI`.
- **Recommendation:** A. It keeps both environments in `Firna-AI` and
  needs no code. B is the code choice when both environments must run the
  same repositories.
- **Action:** ask the user, because the options trade the repositories
  and the labels of each environment, and no option is clearly best.
