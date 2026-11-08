# Waiting adds stay hidden after access ends

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Large.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-06, from `conductor/mvp-m19-ui`, PR #36, the options
  check of recovery item 5, which came from review finding 1. The check
  found that the requirements already settle the page order, so the item
  needs no user decision.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `ts/app/src/routing/landing.ts`, the removed-App and
  ended-access steps of `landingDecision`, `ts/app/src/view/access.ts:142`,
  `appRemovedPage`, and `:176`, `accessEndedPage`,
  `crates/ciproxy-api/src/handlers/link_views.rs:55`, the claim view of a
  browser-only sign-in, `docs/mockups/src/screens/auth/app-removed.tsx:18`,
  and `docs/mockups/src/screens/auth/no-role.tsx:50`, `LeftBody`.
- **Reason:** The state needs a lost access and a waiting add at once, and
  its fix needs a new mockup state first, then an API change.

- **Terms:**
  - **Claim:** a row of `installation_claims` for one App install. It names
    the person who started the install, and the workspace and the
    organization after the install links.
  - **Add:** an install that an Admin starts with "Add an organization". It
    joins only a workspace where its person is an Admin, and it never makes
    a workspace.
  - **Direct install:** any other install. It makes a new workspace, or it
    returns a removed organization to the workspace that it left.
  - **Browser-only sign-in:** a sign-in with no retained private workspace
    session, because the person has no role and no request left.
    `Principal::browser_only` in `crates/ciproxy-api/src/access/mod.rs`
    reports it.
  - **Link state:** the `state` of an `OrganizationLinkView` in
    `crates/ciproxy-api/src/wire/links.rs`, such as `waiting_for_admin` or
    `reauthorization_required`.
- **Background:**
  - `docs/protocol/app-removal.md`, in "Removing the App", requires the
    dashboard to say that the App was removed, and how to install it again.
  - `docs/protocol/workspaces.md`, in "Choose the workspace", covers a
    checked add when the person is an Admin of no workspace. The
    organization joins no workspace and waits. The page asks the person to
    get the Admin role from an Admin of a workspace, and to check again.
  - The same doc, in "A request that an owner approves later", says that an
    approved request waits until its person signs in again. Then the check
    and the choice run as on a return.
  - Each sign-in deletes each role of the person in a workspace where the
    person is a member of no organization that still has the App. See
    `save_on` in `crates/ciproxy-store-pg/src/dashboard/identity.rs:92`.
  - A person with no role keeps no stored session. See `prune_on` in the
    same file, and `establish` in
    `crates/ciproxy-store-pg/src/dashboard/sessions.rs`.
  - So a browser-only person is an Admin of no workspace. `prove_links` in
    `crates/ciproxy-api/src/access/links.rs` checks each unlinked claim
    again at each sign-in. It offers only the workspaces where the person
    is an Admin, from `eligible_workspaces`, so it offers none.
  - The add can join a workspace only after the person has an Admin role
    again. So the old-access recovery comes first, and the waiting add
    stays visible beside it. Both requirements then hold, and no order is
    left for the user to choose.
  - `session` in `crates/ciproxy-api/src/handlers/sessions.rs` gives a
    browser-only sign-in the state `app_removed` when the sign-in found an
    organization that was removed in the last 30 days. Otherwise it gives
    `access_ended` when the person had access before, else
    `install_required`. It gives no workspaces.
  - The claim view of a browser-only sign-in is the first branch of `view`
    in `link_views.rs`. It gives `waiting_for_admin` only to a claim with
    four facts: its GitHub event arrived, its check passed, it has no saved
    workspace, and it has no offered workspace.
  - That view gives every other claim `reauthorization_required`. It gives
    no workspace data.
  - Linking saves the workspace and the organization of a claim together,
    at `crates/ciproxy-store-pg/src/installation_links/activation.rs:144`.
    So a completed historical claim never reads `waiting_for_admin`. It
    reads `reauthorization_required`.
  - So `reauthorization_required` mixes several cases. Examples are a
    completed historical claim, a claim that waits for its GitHub event,
    and a claim whose check failed.
  - The view ignores whether a claim is Direct. A Direct claim between its
    event and its registration has no workspace yet, so it reads
    `waiting_for_admin` for a short time. A Direct install never needs an
    Admin workspace.
  - For a sign-in with a workspace session, the rest of `view` already
    gives the true progress, from `InstallationClaim::recovery`.
  - `landingDecision` in `ts/app/src/routing/landing.ts` opens
    `/app-removed` or `/access-ended` for these states before it looks at
    any older claim.
    Only two steps come first: a session that asks for a new GitHub
    sign-in, and the open claim of a new install that this tab started.
  - `appRemovedPage` and `accessEndedPage` in `ts/app/src/view/access.ts`
    show no claim.
  - `appRemovedPage` offers only Install on GitHub. It has no Check again.
  - `accessEndedPage` offers Check again and Install on another
    organization. Its Check again opens a new GitHub sign-in through the
    `reauthorization_url` of the session, in `checkAgainTarget` of
    `access.ts`. That sign-in runs the claim check again.
  - `ts/app/src/routing/continuation.ts` trusts a claim of a browser-only
    sign-in only when this tab started its install. Its comment says that
    these claims show old completed installs as open.
  - The return page of Add an organization gives each claim one section,
    which `ts/app/src/view/add-organization.ts` builds. In its
    `waiting_for_admin` case, the section says "GitHub installed
    ciproxy on `<organization>`. To add it, ask an Admin of your workspace
    for the Admin role, then check again." It has one Check again.
  - The removed-App mockup is `docs/mockups/src/screens/auth/app-removed.tsx`,
    the `removed` entry of
    `docs/mockups/src/entries/app/github-app/states.mockup.tsx`. It shows
    only the removal and how to install the App again.
  - The ended access is the `left` state of
    `docs/mockups/src/screens/auth/no-role.tsx`, in
    `docs/mockups/src/entries/app/access/index.mockup.tsx`.
  - Neither mockup shows a waiting add.
  - The browser smoke cases `root-removed-old-claim` and
    `root-removed-old-outcome`, in `ts/app/scripts/browser-review-cases.ts`,
    expect the removed-App page for a session with an old claim that reads
    `reauthorization_required`.
- **Problem:** An earlier access ended or the App left, and an add of the
  same person waits for an Admin workspace. The landing then opens the
  removed-App or ended-access page, which shows no claim, so the waiting add
  and its request for the Admin role stay hidden. The app cannot show every
  claim that looks open, because completed historical claims also read
  `reauthorization_required`.
- **Impact if no action:** The person never learns that the add waits, or
  that it needs the Admin role and a new check. Showing every claim that
  looks open first would instead hide the required removal page, and ask
  for a new sign-in for old completed claims.
- **Root question:** What should a person without a workspace session see
  when old access ended and an add also waits?
- **Options:**
  - **A.** Keep the removed-App or ended-access page as the landing. Show on
    it each claim that reads `waiting_for_admin`, with the return section
    and one Check again. Add this state to both mockups first. Change no
    API. Cost: a waiting add whose check failed stays hidden, because it
    reads `reauthorization_required`. A Direct claim between its event and
    its registration can show as waiting for a short time. Effort: Medium.
  - **B.** Apply A first. Then make the claim view of a browser-only sign-in
    give the true progress, still with no workspace data. A linked claim
    reads as final, or the view leaves it out. A claim that waits for its
    event reads `confirming_installation`. A Direct claim never reads
    `waiting_for_admin`. Then the old-access page also shows each open
    claim whose check failed. Cost: a Rust change, HTTP tests with the real
    store, and regenerated API types. Effort: Large.
  - **C.** Correct the claim view as in B first. Only then show the open
    claims on the old-access page, with an action for the waiting add.
    Cost: the work of B, but the waiting add stays hidden until the API
    change lands. Effort: Large.
  - **D.** Show the waiting add first, and keep the old-access recovery as
    secondary content. It needs the true progress of B and a combined
    mockup. Cost: the page leads with an add that cannot finish until the
    person has an Admin role again. Effort: Large.
  - **E.** Build a new combined recovery page for all access and install
    states. Cost: a new screen and flow for the same facts. Effort: Large.
- **Left out:**
  - Change nothing: it fails the goal, because no page asks for the Admin
    role and a new check.
  - Show every claim that looks open first: it treats old completed records
    as current work, and it hides the removal page.
  - Hide a claim whose login matches the removed organization: it guesses,
    because a new install on the same organization matches too.
  - A landing page that the API chooses: A is better on every point. It
    uses the same facts with no Rust change.
- **Wider fix:** B. The claim view then gives the true progress, so no page
  has to guess which open claim is real.
- **Previous examples:** `ts/app/src/routing/continuation.ts` already works
  around the same masked view. For a browser-only sign-in, it trusts a
  claim only when this tab started its install.
- **Recommendation:** B. The person needs an Admin role again before the add
  can finish, so the old access leads, and the waiting add stays visible.
  Its first step, A, shows the add before the API change lands.
- **Action:** fix after merge, apply B, because the state is rare, and its
  mockup state must come first. The requirements above already settle the
  order, so the item needs no user decision.
  - Mockups first. Add the state to the removed-App mockup and to the
    `left` state of the no-role mockup. The old-access content leads. One
    section below it shows the waiting add, with one Check again.
  - Show no workspace data in this state. The session names no workspace,
    so check the reused copy "ask an Admin of your workspace".
    `docs/protocol/workspaces.md` says "an Admin of a workspace".
  - Run `npm --prefix ts run mockups:check`. Inspect the new states at
    desktop widths of 1440 and 720 pixels, and a mobile width of 390
    pixels.
  - For the app part of A, show on `/app-removed` and `/access-ended` each
    claim of `organization_links` that reads `waiting_for_admin`. Reuse
    the return section. Keep the landing order.
  - Show no claim that reads `reauthorization_required` until the API part
    lands. Give each page one Check again for the waiting add, through the
    `reauthorization_url` of the session. The ended-access page already has
    one. The removed-App page gets its first.
  - Add page-model cases to `ts/app/_tests_/access.test.ts` for a removed
    App and for an ended access. Give each case a `waiting_for_admin` claim
    and a `reauthorization_required` claim.
  - Add a browser smoke case with a fixture session in this state, at 1440,
    720, and 390 pixels. Keep `root-removed-old-claim` and
    `root-removed-old-outcome` passing.
  - For the API part of B, change only the first branch of `view` in
    `link_views.rs`. Give the true progress, and still no workspace data.
    Keep every authorization check. The change gives true facts, not a
    role.
  - A claim whose check failed keeps `reauthorization_required`. Choose
    whether a linked claim reads as final or is left out, and record the
    call in the change.
  - Add HTTP tests with the real store in `crates/ciproxy-api/tests/http/`.
    Cover a browser-only sign-in with a linked historical claim, a claim
    that waits for its event, a Direct claim before its registration, a
    checked Add with no Admin workspace, and a failed check.
  - Run `npm --prefix ts run api:generate` when the OpenAPI document
    changes. Do not edit the generated types by hand.
  - After the API part, show each open claim on the old-access page, also a
    claim whose check failed. Then review the rule and the comment of
    `ts/app/src/routing/continuation.ts`, because both assume the masked
    view.
  - No protocol doc changes. Update `ts/app/README.md` and
    `crates/ciproxy-api/README.md` when an entry point or a public
    behaviour changes.
