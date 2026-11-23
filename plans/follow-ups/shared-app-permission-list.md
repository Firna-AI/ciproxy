# Three copies of the App permission list drift

- **Severity:** Low. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-18, from round 2 of the final review of 17 November
  2026 of the branch `calummoore/ci-proxy-load-balancer`, finding 1204. Its
  fix applied option A, and option C waits here. The incomplete reason of
  "Actions" read comes from the check of finding 1207.
- **Source:** Review.
- **Category:** Code structure.
- **Location:** `ts/app/src/view/access.ts:62`,
  `docs/mockups/src/fixtures/github-app.ts:13`,
  `ts/site/src/data/workflow.ts:11`, and
  `ts/site/src/content/docs/docs/reference/github-app.md:11`, and
  `docs/protocol/security.md`, "GitHub App permissions".
- **Reason:** Option C adds a workspace package and changes three
  consumers, which is more than the correction of the Members words needs.

- **Terms:**
  - **App permission list:** the permissions that the ciproxy GitHub App
    asks for, each with its access level and the reason that ciproxy needs
    it.
  - **Install page:** the dashboard page `/install`. Under "The app asks
    for", it lists the permissions before the person goes to GitHub.
- **Background:**
  - `docs/protocol/security.md`, "GitHub App permissions", states four
    permissions and the reason of each: "Self-hosted runners", read and
    write; "Metadata", read; "Members", read; and "Actions", read.
  - Three packages keep their own copy of the list:
    - The dashboard: `appPermissions` in `ts/app/src/view/access.ts:62`,
      one string for each permission, with its access level and its
      reason. The install page shows it.
    - The mockups: `appPermissions` in
      `docs/mockups/src/fixtures/github-app.ts:13`, with the name, the
      access level, and the reason. The install mockup lists it, and the
      landing mockup counts it.
    - The site: `appPermissions` in `ts/site/src/data/workflow.ts:11`,
      with the names and the access levels. The landing page counts it.
      The docs page "GitHub App permissions", in
      `ts/site/src/content/docs/docs/reference/github-app.md:11`, has its
      own table with the reasons.
  - The reasons drifted twice. Both times the dashboard list and the
    mockup list had the same wrong words:
    - A review on 17 November 2026 found the old reason of "Actions": the
      branch of each run. Its fix gave both lists "to read the event and
      the head repository of each run".
    - A second review on 17 November 2026 found the old reason of
      "Members": "see who belongs to your organization, so that ciproxy can
      give roles". It suggested a member list, and it named no ownership
      check. Its fix gave both lists and the row of the site docs table
      "to check your own organization memberships and ownership".
  - The control plane also uses "Actions" read for two more reads:
    - The GitHub cache entry list. `CacheEntries::cache_entries` in
      `crates/ciproxy-github/src/rest/cache_metadata.rs` reads
      `repos/{repository}/actions/caches`. "Restore order" in
      `docs/protocol/cache.md` needs these entry facts.
    - The job-time recovery. The timing repair reads
      `repos/{repository}/actions/runs/{run_id}/jobs` in
      `crates/ciproxy-github/src/rest/job_times.rs`, to save the real start
      and finish of a job.
  - `crates/ciproxy-github/run-metadata.md` describes both reads with the
    "Actions: read" permission. But `docs/protocol/security.md` and the
    table of the site docs page name only the event and the head
    repository as the reason of "Actions", and the dashboard list and the
    mockup list give the same incomplete reason.
  - `plans/follow-ups/job-facts-from-the-hook.md` asks whether the App can
    drop the "Actions" read permission. That change would edit each copy
    again.
  - Some sentences sum up the permissions in prose, and they stay
    hand-written: `ts/site/src/components/marketing/Security.astro:28`,
    `docs/mockups/src/screens/marketing/security.tsx:26`, and the privacy
    text in `ts/site/src/data/legal-privacy.ts:43`.
  - The mockups depend only on `@ciproxy/tokens`. That package holds only
    the colours, fonts, and radii of Graphite, the product look.
  - The site docs page `ts/site/src/content/docs/docs/reference/plans.mdx`
    already builds its table from data.
- **Problem:** Each package keeps its own copy of the App permission list.
  A change of a permission or of its reason must reach three places, and
  the copies drifted twice. The protocol doc and every copy also give an
  incomplete reason for "Actions" read.
- **Impact if no action:** The next permission change can leave a copy
  with an old reason. The install page or the docs then give customers a
  wrong reason for a permission. Today they under-state what "Actions"
  read is for.
- **Root question:** How should the install page, its mockup, and the site
  docs give a correct reason for each GitHub App permission, also after the
  next permission change?
- **Options:**
  - **A.** Correct the words of each copy when they drift. The fix of the
    Members words did this. Effort: Small.
  - **B.** Show only the names and the access levels of the permissions,
    with a link to the docs page "GitHub App permissions". Remove the
    reasons from the install page and its mockup. Effort: Small.
  - **C.** Make one shared permission list:
    - Put it in a small private workspace package beside
      `@ciproxy/tokens`. It exports the name, the access level, and the
      reason of each permission, with the facts of
      `docs/protocol/security.md` in plain product words.
    - The dashboard list, the mockup fixture, and the site import it. The
      site docs table becomes MDX, as `reference/plans.mdx` builds its
      table from data.
    - Give "Actions" its complete reason in the list: the run facts of each
      job, the GitHub cache entry list, and the job-time recovery. Correct
      the reason in `docs/protocol/security.md` in the same change.
    - The prose sentences above stay hand-written.
    - Cost: a new package with its README, scripts, and gate coverage, and
      three consumers to change.
    - Cost: the mockup and the dashboard then share their words. The
      `mockups` skill asks that a mockup changes before the product. The
      same review left out a shared copy source of the claim words of the
      dashboard and the mockups for this reason. The permission list is a
      fact of the protocol doc, not a design choice, but a change of the
      list then changes the mockup and the product in one step.
    - Effort: Medium.
  - **D.** Remove the list from the install page and its mockup. Link to
    the docs page "GitHub App permissions". GitHub's own install screen
    shows the exact permissions. Cost: the person sees no reasons before
    the screen of GitHub. The install design of the mockups changes, so
    the user decides. The mockup list stays, because the landing mockup
    counts it. Effort: Small.
- **Left out:**
  - A change of a permission itself: it breaks the decision of 8 November
    2026 to add "Members", read, which `docs/protocol/security.md` keeps in
    force.
  - A test or a check that compares the copies: it fails the goal. The
    dashboard list and the mockup list had the same wrong words, so a
    comparison passed. A check across three packages is also a new check,
    which needs the user.
  - An API field that sends the list: the reasons are fixed product text.
    A field adds a wire contract and still leaves the mockup and site
    copies.
  - The list in `@ciproxy/tokens`: that package holds only the colours,
    fonts, and radii of Graphite.
- **Wider fix:** C. The `documentation` skill asks for each fact in one
  place. That rule did not keep the lists the same two times, because the
  dashboard list is product code, and each package keeps its own copy.
- **Previous examples:** The reasons of "Actions" and "Members", both on
  17 November 2026, as the background says.
- **Related:** `plans/follow-ups/job-facts-from-the-hook.md`. Its change of
  the permissions edits each copy of the list. With C, that change edits
  one list. The complete reason of "Actions" names the cache entry list and
  the job-time recovery, which also block that drop. Neither item waits for
  the other.
- **Recommendation:** C. One list removes the copies, so the next
  permission change edits one place, and no copy keeps an old reason.
- **Action:** fix after merge, apply C, because C adds a package and
  changes three consumers.
