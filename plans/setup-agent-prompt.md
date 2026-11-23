# Setup agent prompt

Status: Active. On branch `calummoore/copying-llm-xioi`, no pull request
yet. The pull request that completes the work deletes this plan, as the
`plans` skill says.

The last step of Setup, "Run the first job", shows a diff that changes
`runs-on: ubuntu-latest` to `runs-on: ciproxy`. Its Copy button writes only
the new YAML lines. A coding agent, such as Claude Code or Cursor, cannot
act on that text alone: it does not say which files to edit, which jobs to
change, or when to stop. This change adds a second button, "Copy prompt for
your agent". It copies a plain-text instruction that moves the jobs of a
repository to ciproxy labels of the right size. The change does not cover
the docs site, an `llms.txt` file, or a pull request that ciproxy opens
itself.

Decisions, 23 November 2026:

- **How a user hands the first-job change to their coding agent.** The user
  chose A, a copy-prompt button with the prompt text in the app. The
  alternatives were B, tabs on the code block with a visible prompt, and C,
  option A with agent-readable docs on the site. Left out: change nothing,
  because the copied YAML is not an instruction. A pull request from
  ciproxy, because the App would need write access to code. Editor deep
  links, because each works for one tool only. An MCP server or a CLI,
  because it is a large new surface for a one-line change. Reason: A meets
  the goal with the least UI.
- **What the prompt covers.** The user added two requirements in the same
  answer. The prompt moves every job of the repository where a ciproxy label
  applies, not one job. It picks the label from the size of the current
  runner of each job.
- **Self-hosted runners and other runner services.** The user decided that
  the agent asks the user before it changes such a job, instead of keeping
  it unchanged. Reason: the user may want to move these jobs to ciproxy
  too, and only the user knows.
- **A suggested label in each question.** The user decided that each
  question of the agent suggests a ciproxy label for the user to approve.
  The agent gives the label and why it fits, from what it can find, such as
  a size in the runner name. Reason: an approval is faster for the user
  than a choice from nothing.

Design calls, my calls, 23 November 2026:

- **The labels come from the API.** The prompt lists each label of the
  workspace with its platform, vCPU, memory, and free disk, from the
  `labels` field of the Setup response. Reason: the Setup API already
  returns them, custom labels can exist, and Setup shows only API facts.
- **macOS labels only with a Mac.** The prompt lists macOS labels only when
  a Mac of the workspace can start VMs. Reason: a macOS label runs on the
  customer's Macs only, so a moved macOS job with no Mac waits forever.
- **Only labels that can run.** The prompt leaves out a Linux label that
  allows neither own machines nor Boat. Reason: its jobs never start.
- **The size rule.** For each job, the agent picks the smallest label of
  the same platform with at least the vCPU and memory of the current
  runner. It uses the size that GitHub documents for that runner on a
  private repository. Reason: a smaller VM can make a job slow or fail, and
  a much larger one takes slots that other jobs need.
- **Jobs that stay.** The agent keeps a job unchanged when no label is
  large enough, or when it runs on Windows or on Arm. The agent lists each
  kept job with its reason. Reason: ciproxy cannot run such a job, and a
  wrong move makes it fail or wait.
- **Ask when the size is unknown.** When the agent cannot find the size of
  the current runner, it asks the user before it changes the job, with a
  suggested label. Reason: the user knows the size, and this matches the
  user's decision for self-hosted runners above.
- **Matrix and reusable workflows.** The agent changes `runs-on` values in a
  `matrix` and in reusable workflows of the same repository in the same
  way. Reason: these hold the runner of many jobs.
- **Private repositories only.** The prompt tells the agent to stop when the
  repository is public. Reason: ciproxy never runs jobs of public
  repositories, as "Move your workflows" in the docs says.
- **Commit and push.** The prompt tells the agent to commit and push as the
  repository usually does. Reason: Setup waits for a job, and the push
  starts one. The repository's own rules decide the branch.
- **The visible step stays.** The step text and the diff stay as they are,
  for people who edit the file by hand. Reason: the user asked for a prompt
  option, not a change to the manual path.

Contract owners:

- The [app README](../ts/app/README.md), section on Setup.
- The [mockups](../docs/mockups/README.md), onboarding first-job entry.

## Milestone 1: Mockups

Tags: mockup

Depends on: none

The first-job step of the onboarding mockup gets the second button below
the code block.

- [ ] Add "Copy prompt for your agent" to `FirstJobStep` in
      `docs/mockups/src/screens/onboarding/step-bodies.tsx`, in both waiting
      states of the step.
- [ ] Run `npm --prefix ts run mockups:check`. Smoke-test the first-job
      entry.
- [ ] Update the design and the TODOs of this plan for each problem that the
      mockups show.

## Milestone 2: Documentation

Depends on: 1

No protocol doc states the Setup screen, so no protocol doc changes.

- [ ] Add one sentence on the agent prompt to the Setup section of
      `ts/app/README.md`.

## Milestone 3: The prompt and the button

Tags: ui

Depends on: 2

- [ ] Add `ts/app/src/view/setup-agent-prompt.ts`. It builds the prompt
      text from the labels of the Setup response and the machines, under
      the design calls above.
- [ ] Add tests in `ts/app/_tests_/setup-agent-prompt.test.ts`: the default
      labels, a custom label, macOS labels with and without a Mac, and a
      label that can run nowhere.
- [ ] Add a labelled copy button for the prompt to `JobStep` in
      `ts/app/src/screens/setup/step-bodies.tsx`. Reuse `useCopy` and the
      failure notice of `CodeBlock`.
- [ ] Check the step in a browser at desktop and phone widths. Save the
      screenshots under `.context/setup-agent-prompt/`.

## Milestone 4: Verification, commit, review, and pull request

Depends on: every other milestone

- [ ] Run the targeted tests and the complete gate named in `AGENTS.md`.
- [ ] Commit with Conventional Commits and push the branch.
- [ ] Review the complete local diff against `origin/main` with the
      `implementation-review` skill after the push. Report the findings,
      then apply the fix rule: fix the `fix now` findings, record
      follow-up items under `plans/follow-ups/` for the rest, re-review
      once, and report.
- [ ] Delete this plan when the `plans` skill says, under "Deleting the
      plan".
- [ ] Open the pull request. Its description gives the decisions and the
      design calls of this plan with their dates, a permalink to this plan,
      and the follow-up items that the change adds.
