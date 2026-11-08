# Jobs mockups name facts that the API lacks

- **Severity:** Low. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-08, from `conductor/m21-jobs-savings-and-billing-ui`,
  the handoff of the Jobs pages.
- **Source:** Implementation.
- **Category:** Mockup.
- **Location:** `docs/mockups/src/screens/jobs/live-notices.tsx:62`,
  `docs/mockups/src/screens/jobs/live-notices.tsx:66`,
  `docs/mockups/src/screens/jobs/live-notices.tsx:97`,
  `docs/mockups/src/screens/jobs/job-detail.tsx:67`,
  `crates/ciproxy-api/src/wire/jobs.rs:235`,
  `crates/ciproxy-api/src/wire/job_overflow.rs:11`,
  `crates/ciproxy-overflow/src/service/start.rs:173`,
  `crates/ciproxy-overflow/src/service/provider_preparation.rs:56`, and
  `crates/ciproxy-github/src/installation_state.rs:82`.
- **Reason:** The choice decides which side of a mockup and product
  mismatch is right, and that choice belongs to the user.

- **Terms:**
  - **Known-data scene:** a mockup scene of the fixture story, which shows
    each page with every fact.
  - **Current-contract scene:** a mockup scene that shows the same moment
    with only the facts that the API gives, as under
    `docs/mockups/src/entries/app/jobs/current-contract/`.
  - **Largest-size fallback:** the setting "Run on the largest Boat size",
    which runs a job on the largest Boat size when no size fits its label.
  - **Size choice:** the Boat size that the overflow service picks when it
    prepares a sandbox, and its kind: a size that fits the label, or the
    largest-size fallback.
- **Background:**
  - The known-data Jobs scenes name three facts that the API does not
    give:
    - The person and the day of a suspension: "Tom Becker suspended it on
      GitHub on 19 November", at `live-notices.tsx:97`.
    - The reason of a Boat refusal, with an "Open Boat" action: "The start
      limit of your Boat plan is full until 10:13", at
      `live-notices.tsx:62-66`.
    - The fit of the Boat size that ran a job: "The cheapest Boat size that
      fits the label and the image. Boat bills this size.", at
      `job-detail.tsx:67`.
  - Suspension: `JobVisibility` at `wire/jobs.rs:235` gives the
    organization, `suspended`, `removed`, and a GitHub URL. The
    installation lookup at
    `crates/ciproxy-github/src/installation_state.rs:44-49` reads
    `suspended_at` from GitHub, and keeps only a flag at `:82`. The code
    trusts that lookup over the order of the events, at `:15`. The
    installation webhook also names a `sender`, the owner or member who
    caused the event, by a numeric ID only, at
    `crates/ciproxy-github/src/app/webhook.rs:63-67` and `:80`. A
    suspension saves no person and no day. `docs/protocol/app-removal.md:98`
    asks for a notice that names the organization and links to GitHub, and
    the product meets it.
  - `docs/protocol/security.md:253-254` keeps the record of a person only
    while the person has a role or an open request for access. The person
    who suspended the App can have no role in the workspace.
  - Refusal: `SandboxStartRefusal` of the `sandbox-interface` crate, which
    the root `Cargo.toml` pins, gives a typed reason with a temporary, a
    plan, or a billing refusal. A temporary reason is, for example, a start
    rate, no ready machine, no capacity, or a concurrency limit of the
    account. The overflow service keeps only the retry time of a temporary
    refusal, the size name of a plan refusal, and a flag for a billing
    refusal, at `start.rs:173-191`. `JobOverflow` at
    `wire/job_overflow.rs:11` has flags for a rejected key, required Boat
    billing, and the spend limit, and each has its own notice. For other
    refusals, it gives only `retry_at` and `refused_sizes`. No API field
    and no overflow code holds an address of the Boat account.
  - Fit: `JobView` gives the Boat size name, the actual size, and the
    retained request of the label. `docs/protocol/overflow.md` says that
    the job page names the Boat size that ran the job, because Boat bills
    that size, and that the largest-size fallback can make it smaller than
    the label. The scaler gives such a start the reason
    `StartReason::LargestBoatSize`, at
    `crates/ciproxy-scaler/src/actions.rs:75`, but no store keeps it.
  - The overflow service then makes the size choice again when it prepares
    the sandbox, with the current catalog, the current label setting, and
    the current size refusals, at `provider_preparation.rs:31-59`, called
    at `crates/ciproxy-overflow/src/service/preparation.rs:93`. It drops
    the kind of that choice at `:56`. So the start reason of the scaler
    can disagree with the size that ran. The start record keeps the chosen
    size, and the job view query already joins that record, at
    `crates/ciproxy-store-pg/src/job_views.rs:112-115` and `:128`.
  - The product shows only the facts of the API: "The GitHub App is
    suspended on `<organization>`" with "Open GitHub", "Boat refused new
    sandboxes" with the retry time and no action, and "Boat bills this
    size." This copy is in `ts/app/src/view/jobs-notices.ts`,
    `ts/app/src/view/jobs-boat-notices.ts`, and
    `ts/app/src/view/jobs-detail.ts`, and the current-contract scenes show
    it. Each sentence is true.
- **Problem:** The known-data scenes and the product disagree on these three
  facts.
- **Impact if no action:** The mockups keep two versions of these notices,
  and a person cannot see who suspended the App, why Boat refused, or that
  a job ran on a smaller size than its label.
- **Root question:** Which of these facts must the Jobs pages name?
- **Options:**
  - **A.** Accept the current-contract copy as the spec, and change the
    known-data scenes to it. Effort: Small.
  - **B.** Add all three facts to the API. The installation lookup reads
    the day and the person of the suspension. The GitHub REST reference,
    checked on 8 November 2026, gives `suspended_at` and `suspended_by` on
    the installation object:
    [Get an installation for the authenticated app](https://docs.github.com/en/rest/apps/apps#get-an-installation-for-the-authenticated-app).
    The overflow service keeps the typed reason of each refusal, and the
    API gives an address of the Boat account, which needs a source first.
    The kind of the size choice is saved at the preparation, as in D. B
    must resolve one condition first: the person can have no role in the
    workspace, and `docs/protocol/security.md:253-254` keeps the record of
    a person only while a role or an open request exists. The work spans
    the GitHub installation state, the overflow state, the start record,
    the wire, the generated types, the app, and the mockups. Effort:
    Large.
  - **C.** Save only the `StartReason` of the Boat start, so the job page
    can name the largest-size fallback, and accept the current-contract
    copy for the other two. Risk: the start reason is the intent of the
    scaler, and the size choice at the preparation can differ from it, so
    the saved reason can disagree with the size that ran. Effort: Medium.
  - **D.** Keep the kind of the size choice with each Boat start. When the
    overflow service picks the size at the preparation, it saves whether
    the size fits the label or is the largest-size fallback, next to the
    chosen size in the start record. The job view gives the kind to the
    job page, which words the caption of the Boat size from it. Accept the
    current-contract copy for the suspension and the refusal. This drops
    the assumption of C that the fit comes from the start reason of the
    scaler, because the size that ran comes from the later size choice.
    Cost: a field in the stored start record and on the wire. Older starts
    have no value, so they keep "Boat bills this size.". Effort: Medium.
  - **E.** Do D. Also keep the typed reason of a temporary Boat refusal
    next to `retry_at`, and word it in the live notice, such as a full
    start limit of the Boat plan, or no free Boat capacity. Show no "Open
    Boat" action, and leave out the person and the day of a suspension.
    This drops the shared assumption that the three facts go together:
    each fact has its own source, cost, and risk. Cost: a field in the
    stored overflow state, a wire field, and copy for five reasons.
    Effort: Large.
- **Left out:** A fit that the page computes from the retained request and
  the actual size, because the page has neither the image root nor the
  Boat sizes of that moment. So it cannot tell the cheapest fit from the
  fallback in every case.
- **Wider fix:** None. Each fact needs its own source.
- **Previous examples:** None found.
- **Related:**
  - `plans/follow-ups/job-routing-reason-writer.md` A saves each placement
    fact where it becomes known, also the kind of the size choice at the
    preparation. D of this item saves the same fact, so both changes
    should share one stored field. Neither item waits for the other.
  - `plans/follow-ups/billing-banner-facts.md` asks the same question for
    the billing banners. The two answers do not depend on each other.
- **Recommendation:** D. Only the fallback changes what a person
  understands about a job, and the size choice at the preparation is the
  true source of that fact. The first pass recommended C.
- **Action:** ask the user, because the choice decides which side of a
  mockup and product mismatch is right.
