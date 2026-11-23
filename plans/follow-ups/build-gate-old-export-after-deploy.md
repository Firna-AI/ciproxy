# Reloaded tabs keep the old export after deploy

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-22, from pull request #125, the branch
  `conductor/ciproxy-terms-acceptance-and-business-checkout`, review round
  2, a problem outside the scope that the check of finding 7 found.
- **Location:** `ts/app/src/state/build-gate.ts:30`, `createBuildGate`,
  and `:39`, where the first build becomes the baseline.
  `ts/app/src/api/client.ts:111`, where each response gives its build to
  the gate. `deploy/scripts/deploy.py:77`, the build of the API, and `:85`
  to `:96`, the order of the stage step, the Helm upgrades, and the full
  static publication.
  `crates/ciproxy-api/src/router.rs:168`, the `x-ciproxy-build` header.
  `ts/app/src/api/README.md`, the rule under "A new API build".
- **Reason:** The deploy order and the build gate are older than the branch
  that found the case, and no production environment serves people yet.

- **Terms:**
  - **Build:** the identity of one API release, its source commit. Each API
    response names it in the `x-ciproxy-build` header.
  - **Build gate:** `createBuildGate` in `ts/app/src/state/build-gate.ts`.
    It notices a new build and makes the next page change load a new
    document.
  - **Dashboard export:** the static pages and assets of the dashboard,
    built into `ts/app/dist`. A deploy uploads it to the dashboard bucket
    of its environment.
  - **Window:** the time in a deploy after the new API serves and before
    the full publication of the new dashboard export.
- **Background:**
  - The function `main` in `deploy/scripts/deploy.py` first stages the
    dashboard export, at line 85. The stage step creates only the files
    that the dashboard bucket lacks, and it replaces no file. So a new page
    is live before the new API, but each existing page keeps its old
    version. Option B of the deploy-order decision of 23 November 2026
    added this step.
  - Then `main` runs `helm upgrade --install --wait` for the release server
    at line 87, and for the chart `deploy/helm/ciproxy`, which serves the
    API, at line 90.
  - Then the full publication uploads the site export and the dashboard
    export, at lines 94 to 96. It replaces each existing page. So every
    deploy still has the window for the existing pages. The full
    publication after the Helm upgrades is older than pull request #125.
    [Static delivery](../../deploy/static-delivery.md#lifetimes-and-publishing)
    gives the order and its reasons.
  - `deploy/scripts/publish_static.py` uploads every asset before any HTML,
    and it deletes no old file. HTML has `Cache-Control: no-store`. So a
    load of an existing page in the window gets the old HTML and its old
    chunks.
  - `deploy.py` sets the build to the source commit, from
    `CIPROXY_SOURCE_SHA` at line 77. So each deploy of a new commit changes
    the build.
  - The API client gives the build of each response to the build gate, at
    `client.ts:111`. The gate takes the first build that it sees as its
    baseline. A response with another build latches it, and nothing
    unlatches it.
  - After the latch, the next page change loads a new document. So the tab
    never calls a newer API with old types, and the current page keeps its
    input until then. `ts/app/src/api/README.md` states this rule under "A
    new API build".
  - In the window, that new document comes from the old export. Its new
    gate takes the new build as its baseline. When the new export arrives,
    no response names another build, so the gate does not latch again.
  - The window lasts through the uploads of both exports. After a failed
    publication of the dashboard, it lasts until the next successful
    deploy.
  - ciproxy is not in production yet. After the launch, every production
    deploy has the window.
- **Problem:** Every deploy makes the new API live before the new dashboard
  export. A tab that reloads for the new build in that window gets the old
  export, and its build gate takes the new build as its baseline. The gate
  then never latches again, so the tab keeps the old dashboard.
- **Impact if no action:** The tab calls the new API with old types until
  the person reloads it by hand, or a later build latches the gate. Where
  the types changed, its pages can fail.
- **Root question:** How does a tab that reloads for a new build end up on
  the dashboard export of that build?
- **Options:**
  - **A.** The export names its own build, and the build gate compares each
    API response with it:
    - The dashboard export holds the source commit from export time, the
      value that `deploy.py` gives the API as `BUILD_ID`.
    - The gate takes that build as its baseline. A response with another
      build latches it at once. So a tab on the old export loads a new
      document at each page change, until the new export arrives.
    - When the export names no build, as in a local run or in the fixture
      harness, the gate keeps the first build as its baseline.
    - Update the rule in `ts/app/src/api/README.md` and the tests of the
      gate.
    - Cost: while a publication lags or fails, each page change of an old
      tab is a full page load.
    - Effort: Medium.
  - **B.** Publish both exports before the Helm upgrades in every release,
    with a test of the order in `deploy.py`. A reload then always gets the
    new export. A tab that loads the new export in the window calls the
    old API with new types, which the gate does not catch, so each release
    must keep the new dashboard working with the old API. Effort: Medium.
  - **C.** Release the API and the dashboard as a versioned pair. Switch the
    traffic to the pair only when both parts are ready. This removes the
    window, but it changes the deployment infrastructure. Effort: Large.
- **Left out:**
  - Change nothing: the tab keeps the old dashboard with the new API.
  - Reload the tab on a timer after the latch: the reload throws away the
    input that the rule keeps until the next page change.
  - The API compares a build that the dashboard sends in a header: the API
    cannot tell an older build from a newer one, and A needs no API change.
- **Wider fix:** A. The gate then takes its baseline from the export
  itself, so each difference between the export and the API latches it,
  whatever the deploy order, also after a failed publication.
- **Previous examples:** None found.
- **Recommendation:** A. It fixes the baseline where it goes wrong, for
  every deploy order and also after a failed publication, with no change
  to the deployment. C protects more, because it removes the whole window,
  but it is Large.
- **Action:** fix after merge, apply A, because the deploy order and the
  build gate are older than pull request #125, the case is Rare, and the
  fix costs the same after the merge.
