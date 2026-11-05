# Untrusted events can keep Docker disk changes

- **Severity:** Medium. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-05, from pull request #17, review round 1, outside the
  scope. Item 66 of `plans/ciproxy-mvp.md`.
- **Location:** `docs/protocol/cache.md`, "Docker disks".
- **Reason:** Milestone 25 builds the Docker disks later, and the user
  decides whether the rule of item 59 also covers them.

- **Terms:**
  - **Docker disk:** the disk that keeps the Docker data of one repository
    on one own machine between the jobs of the repository.
  - **Trusted event:** one of the 7 events that GitHub lets write
    default-branch caches: `push`, `workflow_dispatch`,
    `repository_dispatch`, `delete`, `registry_package`, `page_build`, and
    `schedule`.
- **Background:**
  - The user answered item 59 with C on 5 November 2026: only the trusted
    events write shared entries in the machine cache.
  - "Docker disks" in `cache.md` says that one job at a time keeps its
    changes to a Docker disk: the first job on the default branch that uses
    the disk. Each other job works on a copy, which ciproxy discards.
  - That rule names the default branch, but no event.
  - Jobs of `issue_comment`, `workflow_run`, and `pull_request_target` also
    run on the default branch ref. A person outside the repository can start
    them.
- **Problem:** A job of an untrusted event can be the job that keeps its
  changes to the Docker disk of a repository.
- **Impact if no action:** Code that a person outside the repository
  started can change the Docker images that later trusted jobs of the
  default branch use. This holds once Milestone 25 builds the Docker disks.
- **Root question:** Which jobs may keep their changes to the shared Docker
  disk of a repository?
- **Options:**
  - **A.** The same rule as item 59: only a job of a trusted event on the
    default branch keeps its changes. Every other job works on a copy.
    Effort: Small.
  - **B.** Keep the current rule: the first job on the default branch keeps
    its changes, whatever its event. Effort: Small.
  - **C.** Only a `push` job on the default branch keeps its changes. A
    dispatch or a schedule then works on a copy, so the disk updates less
    often. Effort: Small.
- **Left out:** No shared Docker disk, where every job works on a copy,
  because it removes the Team feature of item 8.
- **Wider fix:** A. One rule then decides every shared state on a machine.
- **Previous examples:** Item 59, the same question for the machine cache.
- **Recommendation:** A. It keeps one rule for the machine cache and the
  Docker disks, which the user already chose for the cache.
- **Action:** ask the user, because A changes the requirement "Docker
  disks" of `cache.md`, and item 59 named only the machine cache.
