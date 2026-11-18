# App permission reasons live in three copies

- **Severity:** Low. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`: the wider fix C of finding
  1204, split off on 17 November 2026, with the incomplete reason of
  "Actions" read that the check of finding 1207 found.
- **Source:** Review.
- **Category:** Docs or spec.
- **Location:** `ts/app/src/view/access.ts`, `appPermissions`,
  `docs/mockups/src/fixtures/github-app.ts`, `appPermissions`,
  `ts/site/src/data/workflow.ts`, `appPermissions`,
  `ts/site/src/content/docs/docs/reference/github-app.md`, the table under
  "Permissions", and `docs/protocol/security.md`, "GitHub App permissions".
- **Reason:** C adds a new workspace package and changes the app, the
  mockups, and the site.

- **Terms:**
  - **Permission list:** the name, the access level, and the reason of each
    permission of the ciproxy GitHub App.
  - **Install page:** the dashboard page that sends a person to GitHub to
    install the App. It lists the permissions before the person reaches
    the install screen of GitHub.
  - **`@ciproxy/tokens`:** the private workspace package under `ts/tokens`
    that holds the colours, fonts, and radii of the product look.
  - **MDX:** Markdown with components. A Starlight page in MDX can build a
    table from data.
- **Background:**
  - "GitHub App permissions" in `docs/protocol/security.md` states four
    permissions:
    - "Self-hosted runners", read and write, to start a runner for each
      job.
    - "Metadata", read, to list the repositories.
    - "Members", read, to check the person's own organization memberships
      and ownership with that person's App user token. A decision in that
      section chose it over a separate OAuth sign-in.
    - "Actions", read, to read the event and the head repository of the
      run of each job from the GitHub API.
  - Three packages keep their own copy of the permission list:
    - `appPermissions` in `ts/app/src/view/access.ts`, one string for each
      permission, for the install page.
    - `appPermissions` in `docs/mockups/src/fixtures/github-app.ts`, with a
      name, an access level, and a use. The landing mockup counts it.
    - `appPermissions` in `ts/site/src/data/workflow.ts`, with names and
      access levels only, and the table of the docs page
      `ts/site/src/content/docs/docs/reference/github-app.md`.
  - The copies drifted twice. Round 1 of the same review found an old
    reason of "Actions" in the app list and the mockup list. Round 2 found
    that "Members" still said "see who belongs to your organization" there.
    Each time, a direct fix changed the words in each copy.
  - The control plane also uses "Actions" read for two more reads:
    - The GitHub cache entry list. `CacheEntries::cache_entries` in
      `crates/ciproxy-github/src/rest/cache_metadata.rs` reads
      `repos/{repository}/actions/caches`. "Restore order" in
      `docs/protocol/cache.md` needs these entry facts, and it says that
      the control plane reads them with the "Actions" read permission.
    - The job-time recovery. The timing repair reads
      `repos/{repository}/actions/runs/{run_id}/jobs` in
      `crates/ciproxy-github/src/rest/job_times.rs`, to save the real start
      and finish of a job.
  - `crates/ciproxy-github/run-metadata.md` describes both reads with the
    existing "Actions: read" permission.
  - `docs/protocol/security.md` and the table of the site docs page name
    only the event and the head repository as the reason of "Actions". At
    commit `9b094e15`, the app list and the mockup list give the same
    incomplete reason.
  - `plans/follow-ups/job-facts-from-the-hook.md` proposes the next change
    of the list: drop "Actions" read.
  - The site already builds the table of
    `ts/site/src/content/docs/docs/reference/plans.mdx` from data.
  - Some summary sentences stay hand-written:
    `ts/site/src/components/marketing/Security.astro`, the mockup
    `security.tsx`, and the privacy text.
  - The old words of the lists are not a user decision. On 8 November 2026
    the user only chose to add "Members".
- **Problem:** The reasons of the App permissions live in three copies that
  people keep by hand, and two changes already missed copies. The protocol
  doc and the site docs page also give an incomplete reason for "Actions"
  read.
- **Impact if no action:** After the next permission change, such as the
  drop of "Actions" read, customers can see a wrong reason again. Today the
  docs under-state what "Actions" read is for.
- **Root question:** How should the install page, its mockup, and the docs
  give a correct reason for each GitHub App permission, also after the
  next permission change?
- **Options:**
  - **A.** Correct each copy by hand at each change, as the two direct
    fixes did. Effort: Small.
  - **B.** Show only the names and the access levels on the install page,
    with a link to the reasons on the docs page "GitHub App permissions".
    This removes the inline reasons of the install page. Effort: Small.
  - **C.** Make one shared permission list.
    - Put it in a small private workspace package beside `@ciproxy/tokens`.
      It exports the name, the access level, and the reason of each
      permission, with the facts of the protocol in plain product words.
    - The app list, the mockup fixture, and the site import it. The site
      docs table becomes MDX, the way `reference/plans.mdx` already builds
      its table from data.
    - Give "Actions" its complete reason in the list: the run facts of each
      job, the GitHub cache entry list, and the job-time recovery.
    - Correct the reason of "Actions" in `docs/protocol/security.md` in the
      same change. `docs/protocol/cache.md` already states the cache entry
      list.
    - The summary sentences stay hand-written.
    - The list could go in `@ciproxy/tokens` with no new package. But that
      package holds only the colours, fonts, and radii of the product look.
    - C drops the assumption that each package keeps its own copy of the
      list. The source is the code. Its reason holds only in part, because
      the copies drifted twice.
    - Cost: a new package with its README, scripts, and gate coverage, and
      three consumers to change.
    - Effort: Medium.
  - **D.** Remove the list from the install page and its mockup, and link
    to the docs page "GitHub App permissions". The install screen of GitHub
    shows the exact permissions.
    - D drops the assumption that the install page lists the permissions
      before the person goes to GitHub. The source is the install mockup
      and its rationale. No protocol doc requires the list.
    - Cost: the person sees no reasons before the screen of GitHub. It also
      changes the install design of the mockups, so the user decides. The
      mockup list stays, because the landing page counts it.
    - Effort: Small.
- **Left out:**
  - A change of the permission itself: it breaks the decision in
    `docs/protocol/security.md` that the App asks for "Members", read.
  - A test or a check that compares the copies: it fails the goal. The app
    list and the mockup list had the same wrong words, so a comparison
    passed. A check across three packages is also a new check, which needs
    the user. C removes the copies instead.
  - An API field that sends the list: not a real choice. The reasons are
    fixed product text. A field adds a wire contract and still leaves the
    mockup and site copies.
- **Wider fix:** C. The `documentation` skill did not keep these lists the
  same two times. The dashboard list is product code, and each package
  keeps its own copy. C makes the list one fact in one place.
- **Previous examples:** Commit `3428fd3` of 17 November 2026 corrected the
  reason of "Actions" in the app list and the mockup list, one copy at a
  time.
- **Related:** `plans/follow-ups/job-facts-from-the-hook.md`: it proposes to
  drop "Actions" read. With C, that change edits one list. The complete
  reason of "Actions" in C also names the cache entry list and the
  job-time recovery, which block that drop. Neither item waits for the
  other.
- **Recommendation:** C. One shared list makes each reason one fact in one
  place. So the next permission change reaches the install page, the
  mockup, and the site at once. D removes one copy, but it changes the
  install design, which the user decides.
- **Action:** fix after merge, apply C, because C adds a new package and
  reaches the app, the mockups, and the site.
