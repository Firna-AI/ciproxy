# Deploy publishes the dashboard export after the API

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-22, from pull request #125, the branch
  `conductor/ciproxy-terms-acceptance-and-business-checkout`, review round
  2, finding 7.
- **Location:** `deploy/scripts/deploy.py:79` to `:88`, the order of the
  Helm upgrades and the static publication in the function `main`.
  `deploy/scripts/publish_static.py:84`, the order inside one export.
  `deploy/terraform/modules/edge/routing.tf:39`, the `dashboard` path
  matcher. `crates/ciproxy-api/src/handlers/github_setup.rs:36` and
  `crates/ciproxy-api/src/handlers/sessions.rs:150`, two places that send
  people to `/continue`.
- **Reason:** ciproxy has no production environment yet, so the release of
  pull request #125 meets the window only on the main preview, and the fix
  costs the same after the merge.

- **Terms:**
  - **Terms step:** the dashboard page at `/continue`, with the terms
    notice and a "Continue with GitHub" button. Pull request #125 added
    it.
  - **Dashboard export:** the static pages and assets of the dashboard,
    built into `ts/app/dist`. A deploy uploads it to the dashboard bucket
    of its environment.
  - **Window:** the time in a deploy after the new API serves and before
    the new dashboard export is in the bucket.
  - **Main preview:** the `main-preview` environment at
    `br-main.preview.app.ciproxy.dev`. Each push to
    `calummoore/ci-proxy-load-balancer` or `main` deploys it.
- **Background:**
  - The function `main` in `deploy/scripts/deploy.py` deploys one
    environment. It runs `helm upgrade --install --wait` for the release
    server at line 79, and then for the chart `deploy/helm/ciproxy` at
    line 82. That chart holds the control plane, which serves the API.
  - Then it publishes the site export and the dashboard export with
    `deploy/scripts/publish_static.py`, at lines 86 to 88. So the new API
    serves before the new dashboard export is in the bucket. This order is
    older than pull request #125.
  - `publish_static.py` uploads every asset of an export before any HTML,
    and it deletes no old file. So an old page and a new page both find
    their chunks. HTML has `Cache-Control: no-store`, so the edge does not
    cache it, and a published page serves at once.
  - The edge sends each path of the dashboard host that no rule names,
    such as `/continue`, to the dashboard bucket. This is the
    `default_service` of the `dashboard` path matcher in `routing.tf`. The
    route map has no fallback for a page that the bucket does not hold
    yet.
  - Since pull request #125, the API sends people to `/continue`:
    - The setup return redirects there after an install of the GitHub
      App, at `github_setup.rs:36`.
    - The session gives it as the reauthorization URL of a browser-only
      sign-in, at `sessions.rs:150`.
    - Each organization link gives it as its reauthorization URL, at
      `crates/ciproxy-api/src/handlers/link_views.rs:76` and `:138`.
  - The dashboard export before pull request #125 has no `/continue` page.
    So at the first deploy of pull request #125, each environment that
    already serves has no `/continue` page during the window. Each later
    change that adds a page that the API sends people to has the same gap.
  - A person who meets the missing page can retry once the page is
    published. After a failed publication, the next successful deploy
    publishes it. The pending setup of an install lives 10 minutes. After
    that, Continue gives a plain sign-in, and the install page recovers the
    install with "Install on GitHub".
  - ciproxy is not in production. The main preview deploys each push.
    Production deploys only after the launch, which
    `plans/follow-ups/production-launch.md` holds.
- **Problem:** A deploy makes the new API live before the new dashboard
  export. In that window, the new API can send a setup return or a
  reauthorization to `/continue`, which the old export lacks.
- **Impact if no action:** A person who finishes an install or starts a
  reauthorization in the window meets a missing page, and the page stays
  missing until a publication succeeds.
- **Root question:** How does each deploy make a new dashboard page live
  before the new API sends people to it?
- **Options:**
  - **A.** Publish the new dashboard export before the API, for one release
    that adds such a page. Check that the new dashboard works with the old
    API during the change. Each later release that adds such a page needs
    the same step. Effort: Small.
  - **B.** Stage the new dashboard routes and their assets before the API
    rollout. Keep the usual publication order for the existing pages. This
    removes the missing page for each later route. Effort: Medium.
  - **C.** Release the API and the dashboard as a versioned pair. Switch the
    traffic to the pair only when both parts are ready. This removes the
    window, but it changes the deployment infrastructure. Effort: Large.
  - **D.** Keep the current order for the release of pull request #125, and
    do B before the production launch. The branch of pull request #125
    ships with this order. It is safe only while no production environment
    serves people. Effort: Medium.
  - **E.** Publish both exports before the Helm upgrades in every release,
    with a test of the order in `deploy.py`. A tab that loads the new
    export in the window then calls the old API with new types, so each
    release must keep the new dashboard working with the old API.
    Effort: Medium.
- **Left out:**
  - Redirect `/continue` to the old sign-in page while the export lacks it:
    the redirect drops the pending setup.
  - Keep the current order for good: the missing page comes back with each
    later page.
- **Wider fix:** B. The shared deploy then publishes each later route
  before its callers go live.
- **Previous examples:** `deploy/scripts/publish_static.py` already orders
  the uploads inside one export, assets before HTML, so that a page never
  arrives before its chunks. Commit `94afd53` of 4 November 2026 added that
  order, and commit `21f1278` of 5 November 2026 kept it when it split the
  cached assets from the HTML.
- **Related:**
  - `plans/follow-ups/production-launch.md`, work that waits. Apply this
    item before the launch, because after the launch each release can meet
    the window with real customers.
  - `plans/follow-ups/build-gate-old-export-after-deploy.md`, the same
    cause: the deploy order of `deploy.py`. That item holds the other case
    of the window: a tab that reloads for the new build gets the old
    export. B of this item does not fix that case, and C and E would. The
    options of the two items do not conflict.
- **Recommendation:** B. It removes the missing-page window without
  replacing pages that still serve the old API. C best protects the
  codebase, because it removes the whole window, but it is Large.
- **Action:** fix after merge, apply B, because a Medium product bug that
  is Rare waits when its fix does not get harder after the merge, and
  ciproxy is not in production, so the release meets the window only on
  the main preview.
