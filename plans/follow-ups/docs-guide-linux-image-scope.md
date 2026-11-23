# Workflow guide applies Linux image facts to Macs

- **Severity:** Medium. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-23, from the branch `calummoore/diagram-request-3jpv`,
  the check of review finding 3.
- **Source:** Review.
- **Category:** Docs or spec.
- **Location:** `ts/site/src/content/docs/docs/guides/move-your-workflows.mdx:37-46`.
- **Reason:** The branch that found it changes only the page How ciproxy
  works, and these lines were already on `main`.
- **Terms:**
  - **Job image:** the operating system and software that a job VM starts
    with.
- **Background:**
  - The Linux image has the software of the GitHub runner image of
    `ubuntu-latest`, and a Linux job can use the `container:` key, as
    `docs/protocol/agent-images.md` says under "Software".
  - The macOS image has the software of `macos-latest`, and a macOS job VM
    has no Docker, as `docs/protocol/agent-images.md` and
    `docs/protocol/docker-disks.md` under "Docker disks on a Mac" say.
  - The guide Move your workflows says, under "The software of a job", that
    each job starts from the `ubuntu-latest` software. Under "Run steps in
    another image" it says that a job can use the `container:` key on every
    plan.
  - The page How ciproxy works now limits both facts to Linux jobs, since
    the branch that added its diagram. It links to this guide section.
- **Problem:** The guide says that every job, also a macOS job, gets the
  Ubuntu software and the `container:` key.
- **Impact if no action:** A customer can move a macOS workflow and expect
  Ubuntu tools or job containers that a Mac does not have.
- **Root question:** How does the guide say which software and Docker
  features each platform gets?
- **Options:**
  - **A.** Say "Each Linux job" and "A Linux job" in the two sections, and
    add that a macOS job starts from the software of `macos-latest`.
    Effort: Small.
  - **B.** Add a short section on macOS workflows to the guide. It adds a
    section for facts that fit in two sentences. Effort: Small.
- **Left out:** Change nothing, because the guide contradicts the protocol.
- **Wider fix:** None. The `documentation` skill already requires docs to
  agree with the product.
- **Previous examples:** Review finding 3 of the branch
  `calummoore/diagram-request-3jpv`, on How ciproxy works and its mockup.
- **Recommendation:** A. It matches the words of How ciproxy works.
- **Action:** fix after merge, apply A, because the lines are outside the
  diff of the branch that found them.
