# The agent prompt skips repository access checks

- **Severity:** High. **Likelihood:** Common. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-23, from the branch `calummoore/copying-llm-xioi`,
  review round 1, finding 4.
- **Location:** `ts/app/src/view/setup-agent-prompt.ts`, the function
  `agentPrompt`. `crates/ciproxy-api/src/handlers/organizations.rs:41`, the
  repository list of an organization.
- **Reason:** The options have real trade-offs, and no option is clearly
  best, so the user chooses.

- **Terms:**
  - **Agent prompt:** the text that "Copy prompt for your agent" in the
    first-job step of Setup copies for a coding agent, such as Claude Code.
    It tells the agent to move the jobs of the current repository to
    ciproxy labels.
  - **Repository access:** the workspace setting, in Settings under
    Organizations, that allows all private repositories or selected
    repositories of each organization.
- **Background:** ciproxy runs the jobs of a repository only when the
  GitHub App installation can see it, the repository is private, and the
  workspace's repository access allows it. A job of any other repository
  gets no ciproxy runner, and GitHub keeps it queued. The agent prompt
  tells the agent to stop when the repository is public. It says nothing
  about App visibility or repository access.
- **Problem:** The agent can move and push the jobs of a private repository
  that the workspace does not serve. The agent cannot check App access or
  the workspace policy itself: a normal `gh` token cannot read the App
  installation of a repository, and no API that the agent has gives the
  workspace policy.
  The manual path of the same step, "Change runs-on in one workflow file",
  does not say that the repository must be allowed either
  (`ts/app/src/screens/setup/step-bodies.tsx`). The chosen option can
  cover it too.
- **Impact if no action:** The moved jobs stay queued on GitHub with no
  runner, and the first-job step of Setup sees no job.
- **Root question:** How can the prompt avoid moving the jobs of a
  repository that ciproxy will not serve?
- **Options:**
  - **A.** Tell the agent to confirm App access and repository access
    before it edits, and to ask the user when it cannot. Cost: the agent
    cannot verify either fact, so it nearly always asks, and the user often
    does not know. Effort: Small.
  - **B.** Put the active organizations and the access policy of the
    workspace in the prompt. Cost: in the selected mode, Setup holds only
    numeric repository IDs. It cannot see an App selection on GitHub that
    leaves the repository out. Effort: Medium.
  - **C.** Let the user pick an allowed repository in Setup, and name it
    in the prompt. Cost: a new Setup step and a new mockup. Effort: Medium.
  - **D.** Setup reads the repository list of each active organization
    before the button is pressed. `GET
    /workspaces/{id}/organizations/{org}/repositories` returns `full_name`
    and `allowed` for each repository that the App can see. The prompt
    lists the allowed repositories. The agent compares them with
    `git remote`, and stops when the current repository is not in the
    list. Fallback: when the read fails or the list is too long, name the
    organizations and tell the agent to ask the user. Cost: one GitHub
    refresh for each organization, a new store, and its tests. Effort:
    Medium.
  - **E.** Staged move: the agent moves one job, pushes, and checks with
    `gh run view` that a ciproxy runner took it. Only then it moves the
    rest. Cost: a longer agent session and two pushes. A job can also wait
    for a valid reason, such as an image that is still preparing. Effort:
    Small.
- **Left out:** A check of the organization name alone fails when the
  workspace allows selected repositories. A server notice for a queued job
  of a repository that is not allowed misses repositories that the App
  cannot see, and it adds new behaviour that no requirement covers.
- **Wider fix:** D. It gives the agent the server's own answer for each
  repository.
- **Previous examples:** None found.
- **Recommendation:** D. It covers App visibility, the access policy, and
  privacy with the server's own answer. A cannot be verified, and E cannot
  tell "not allowed" from "busy".
- **Action:** ask the user, because the options have real trade-offs in
  cost and coverage, and no option is clearly best.
