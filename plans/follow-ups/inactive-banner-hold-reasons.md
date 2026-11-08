# The inactive banner drops its hold reasons

- **Severity:** Medium. **Likelihood:** Common. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-08, from `conductor/m21-jobs-savings-and-billing-ui`,
  pull request #53, a new problem that the check of owner finding 4 found.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `ts/app/src/view/banner.ts:115`,
  `ts/app/src/screens/workspace-page.tsx:50`, and
  `ts/app/src/api/banner.ts:19`.
- **Reason:** No page of this repository asks for the Organizations scope
  yet, so the fix belongs with the Organizations tab of Settings, which the
  branch that found it does not build.

- **Terms:**
  - **Organizations scope:** the banner read of the Organizations tab,
    `GET /api/workspaces/<id>/banner?page=organizations`. Every other page
    uses the page scope, with no query.
  - **Hold reason:** why an inactive organization starts no jobs. The API
    names `billing_upgrade`, the plan reason, and `admin_approval`, the
    Admin reason.
  - **Banner model:** `bannerModel` in `ts/app/src/view/banner.ts`. It words
    the one banner that the API chose for a page.
- **Background:**
  - `docs/protocol/workspaces.md:166-172`: an organization stays inactive
    while the plan reason or the Admin reason holds, and both can hold at
    once. The plan reason holds while the workspace has more organizations
    than its tier allows. The Admin reason holds when the person who chose
    the workspace was no longer its Admin when the organization joined,
    until a current Admin turns the organization on.
  - `docs/protocol/workspaces.md:203-211`: the Organizations tab marks each
    inactive organization with one line for each reason. The tab also shows
    a banner that names the organization and its reasons, and says that the
    jobs of the other organizations keep running. This banner takes the
    place of the soft-limit banner. The banner of a failed payment takes
    the place of this one.
  - The API chooses that banner for the Organizations scope only, at
    `crates/ciproxy-api/src/handlers/page_context.rs:137-150`. It sends
    `inactive_organizations` with the `id`, the `login`, and the
    `activation` of each inactive organization. `activation` gives
    `reasons`, `added_by` for the person who chose the workspace when the
    API knows it, `upgrade` with the lowest tier that allows the
    organizations, and `can_enable`, `ask_admin`, `admitted`, and
    `preparing`. The generated types are `InactiveOrganization` and
    `ActivationView` in `ts/api/generated/api.d.ts`.
  - The Settings read gives the rows of the tab the same `ActivationView`,
    in `OrganizationView.activation`. It names no organization count of a
    tier.
  - The banner model words this banner at `view/banner.ts:115-126` as
    "`<login>` is inactive", or "`<count>` organizations are inactive", with
    "Jobs keep running in the other organizations of this workspace." It
    reads no field of `activation`, and it gives no action.
  - The app asks for the scope through the `bannerScope` property of
    `WorkspacePageView`, at `workspace-page.tsx:48-55`, which the shell
    gives to the banner. The banner read then sends `page=organizations`,
    at `api/banner.ts:13-19`. No page sets the property, and Settings shows
    a coming-soon page.
  - The mockup words each hold in one `holdReason` helper that the row and
    the banner of the tab share, at
    `docs/mockups/src/components/inactive-banner.tsx:26-35`. The plan hold
    reads "Free includes 1 organization", and the Admin hold reads
    "`<person>`, who added `<login>`, is no longer an Admin". The banner at
    `:59-89` also says when the organization starts its jobs, and it offers
    Choose Team or Turn on. `docs/protocol/workspaces.md:205-207` asks the
    tab for that offer and for Turn on.
  - On 2026-11-08 the user kept the product words of the soft-limit,
    failed-payment, and ended-plan banners. That answer does not cover this
    banner, whose reasons the protocol requires.
- **Problem:** The banner model names the inactive organizations, but it
  drops the reasons that the API sends and the protocol requires.
- **Impact if no action:** When the Organizations tab asks for its scope,
  each visit with an inactive organization shows a banner that does not say
  why, and only the rows say why. Today no page shows this banner.
- **Root question:** How does the banner of the Organizations tab name each
  inactive organization and its reasons?
- **Options:**
  - **A.** Word the reasons in the `inactive_organizations` case of the
    banner model, from `activation.reasons`. Name the person from
    `added_by` for the Admin reason when the API gives it, and the tier
    from `upgrade` for the plan reason. Follow the words of the mockup
    where these fields give the fact. Name no fact that the response does
    not carry, as the header of `view/banner.ts` says for every banner. Keep
    the title, the sentence about the other organizations, and every other
    banner. Add one model test for an organization with both reasons. Risk:
    the rows of the tab can word the same reasons in other words. Effort:
    Small.
  - **B.** Do A through one helper that words each hold reason from
    `ActivationView`. The rows of the Organizations tab use the same
    helper, as `holdReason` serves the row and the banner of the mockup.
    A mockup word that needs a fact outside `ActivationView`, such as the
    organization count of Free, belongs to the work that adds the rows. That
    work settles the words of its rows and its banner together. Effort:
    Small.
  - **C.** Let the Organizations tab word its own banner from its Settings
    read, and remove the Organizations scope and the
    `inactive_organizations` case from the shell banner. Cost: the tab then
    chooses between the payment, ended-plan, inactive, and soft-limit
    banners itself. That repeats the choice of the API, and
    `ts/app/README.md:76-77` says that the API chooses the one banner.
    Effort: Medium.
- **Left out:**
  - No change, because the banner breaks `workspaces.md:208-209` as soon as
    a page asks for the scope.
  - New API fields, because the response already carries every reason.
  - The reasons in the rows only, because the protocol asks the banner to
    name them too.
- **Wider fix:** B. One wording of each reason keeps the banner and the rows
  from giving a different why.
- **Previous examples:** None found.
- **Related:** `plans/follow-ups/generated-query-names.md`, same read: it
  types the `page` query name that the Organizations scope sends. Neither
  item waits for the other.
- **Recommendation:** B. The banner then names the reasons from the fields
  of the API, and the banner and the rows cannot disagree.
- **Action:** fix after merge, apply B, because no page asks for the
  Organizations scope yet, and the rows that share the words belong to the
  Organizations tab, which the branch that found it does not build.
