# Unnamed container steps show as Run script

- **Severity:** Low. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-23, from the branch
  `calummoore/action-cpu-and-memory-usage`, review round 2, the new problem
  of the check of finding 501.
- **Location:** `crates/ciproxy-overflow/src/job_usage_names.rs:19-22`,
  `crates/ciproxy-overflow/src/_tests_/job_usage_tests.rs:213-220`,
  `docs/protocol/job-usage.md:108-109`, and
  `docs/mockups/src/fixtures/job-usage.ts:80-86`.
- **Reason:** To keep a container name, the control plane must trust a mark
  from the VM or keep a name that it cannot tell from a command, and the
  option that keeps no container name narrows a user decision.

- **Terms:**
  - **Container step:** a step with `uses: docker://<image>`, which runs a
    container image as an action.
  - **Script names rule:** the control-plane rule that replaces a step name
    that starts with "Run ", "Pre Run ", or "Post Run " with "Run script",
    "Pre Run script", or "Post Run script", unless the rest names a remote
    action (`crates/ciproxy-overflow/src/job_usage_names.rs`).
  - **Job message:** the JSON message of the job that runner `v2.338.2`
    writes to the Worker log when the job starts
    (`src/Runner.Worker/Worker.cs:101`), after it starts the secret masker
    (`:97`).
  - **Guest name check:** the check that this branch adds to the guest, the
    ciproxy program in the job VM, before its merge. The guest keeps a
    "Run" name only when its rest matches a remote action of the job
    message.
- **Background:**
  - GitHub names a step without a name from its command or its action. On
    23 November 2026, the user chose option C of the step-names decision of
    this branch: store each name, except a name that GitHub makes from a
    `run` command. A "Run" name keeps its text only with a remote action,
    such as `actions/checkout@v4`. Reason: ciproxy stores and logs no code
    of a job.
  - The control plane applies the script names rule before it stores a
    name, because job code controls the VM
    (`docs/protocol/job-usage.md:115-116`). The rule keeps a rest of the
    form `owner/repo[/path]@ref`, or `docker://` and an image
    (`crates/ciproxy-overflow/src/job_usage_names.rs:19-38`).
  - The images pin runner `v2.338.2` (`image/pins.env:11`). The paths that
    start with `src/` are in its repository `actions/runner` at that tag.
  - That runner names an unnamed container step `Run <image>`, without
    `docker://`:
    - `src/Sdk/DTPipelines/Pipelines/ObjectTemplating/PipelineTemplateConverter.cs:601-604`
      strips the `docker://` prefix from the reference.
    - `src/Runner.Worker/ActionRunner.cs:394-398` uses the image as the
      name.
    - The runner test `src/Test/L0/Worker/ActionRunnerL0.cs:265-288`
      expects "Run TestImageName:latest".
  - So the `docker://` branch at `job_usage_names.rs:20` matches no name
    that the runner writes. A bare image, such as `alpine:3.20` or
    `alpine@sha256:…`, also fails the `owner/repo@ref` shape. So each
    unnamed container step shows as "Run script".
  - The examples at `docs/protocol/job-usage.md:109`,
    `docs/mockups/src/fixtures/job-usage.ts:81-82`, and
    `crates/ciproxy-overflow/src/_tests_/job_usage_tests.rs:213-220`
    describe a name that the runner never writes. Before its merge, this
    branch corrects the protocol doc. The doc then says that the control
    plane replaces such a name with "Run script".
  - The guest name check lists each remote action of the job in the form
    that the runner puts in a step name (`ActionRunner.cs:378-398`):
    `owner/repo[/path]@ref` for a repository action, and the image for a
    container action. The control-plane rule stays as a second layer. So
    the guest keeps "Run alpine:3.20", and the control plane replaces it.
  - Job code can change the steps of its own job
    (`docs/protocol/job-usage.md:184-187`). A name that the author chose
    keeps its text. So the script names rule protects the names that the
    runner makes from commands. It cannot stop job code that writes its
    own names.
- **Problem:** Each unnamed container step shows as "Run script", although
  the choice of the user keeps the names of remote actions. To keep such a
  name, the control plane must trust a mark from the guest or accept a bare
  image name. It cannot tell a bare image name, such as `alpine:3.20` or
  `make`, from a command of one word.
- **Impact if no action:** A reader sees "Run script" for each unnamed
  container step, and must open GitHub to learn which image ran.
- **Root question:** How can the job page show the name of an unnamed
  container step, while the stored names hold no command text?
- **Options:**
  - **A.** Keep "Run script" for an unnamed container step, as the
    corrected protocol doc says. Correct the comment of the mockup fixture
    and the test cases, so that they no longer describe a runner name.
    Keep the `docker://` branch, because an author can still write such a
    name. Cost: the names of container steps stay lost, which narrows the
    choice of the user to keep the names of remote actions. Effort: Small.
  - **B.** Send a mark from the guest.
    - The guest name check knows which "Run" names match an action of the
      job. The guest marks each such step in the report.
    - The control plane keeps a marked name whose rest has the shape of a
      container image. An unmarked name keeps the current rule.
    - Cost: a new optional field on the step of `JobUsage` in
      `ciproxy-protocol`, its tolerant decode, and the wire snapshots. The
      control plane then trusts the VM for this case. Job code can set the
      mark, but it can already store any text through a name of its own.
    - Effort: Medium.
  - **C.** Accept a bare image name in the control plane.
    - Keep `Run <rest>` when the rest has the shape of an image reference,
      such as `alpine:3.20`, `ghcr.io/owner/image:tag`, or an image with a
      `@sha256:` digest.
    - Cost: the control plane cannot tell a bare image name from a command
      of one word, such as `make`. So its second layer then keeps each such
      command, and the rule for those names rests on the guest name check
      alone.
    - Effort: Small.
  - **D.** Let the guest write a matched container name in the
    `docker://` form.
    - When the guest name check matches the rest of a "Run" name to the
      image of a container step of the job, the guest sends
      `Run docker://<image>`. The control plane keeps it through its
      `docker://` branch, with no protocol change.
    - Cost: the row then differs from GitHub, which shows "Run
      alpine:3.20", unless the API strips the prefix again. The guest
      changes a name that the runner wrote.
    - Effort: Small.
- **Left out:**
  - The jobs API of GitHub or a read of the workflow file, because the
    Steps decision at `docs/protocol/job-usage.md:79-82` rejects a GitHub
    call for each job.
  - A runner patch that keeps `docker://` in the name, because Boat runs
    the stock runner.
  - Names from GitHub for each page view, because the user chose C over
    that option in the same decision, and
    `docs/protocol/job-usage.md:119-121` records the rejection.
  - A new placeholder, such as "Run container", for a marked step, because
    it still needs the mark of B and hides the image that B can show.
- **Wider fix:** B. The source of a name, not its shape, then decides
  whether it keeps its text, so no shape can stand for both a command and
  an action. The same mark can serve a later name source that the guest
  name check does not see.
- **Previous examples:** None found.
- **Related:**
  - `plans/follow-ups/job-usage-step-name-expressions.md`, a shared part.
    Its option B takes resolved names from GitHub in the control plane,
    where the guest name check does not run, so it needs the source of each
    step from the guest. The mark of B can carry that source.
  - `plans/follow-ups/job-usage-step-links.md`, the same rows. Each "Run
    script" row links to its step on GitHub, and that link opens the run
    page for most jobs. A keeps more rows behind that link.
- **Recommendation:** B. It keeps the exact GitHub name of each container
  step, and only a name that the guest matched to an action of the job
  keeps its text. Choose A if no stored name may depend on a mark from the
  VM.
- **Action:** ask the user, because B makes the control plane trust a step
  mark from the VM, which is new behaviour at a boundary that no
  requirement covers, and A narrows the user's choice of 23 November 2026.
