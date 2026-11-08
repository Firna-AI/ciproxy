# Jobs mockups name facts that the API lacks

- **Severity:** Low. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-08, from `conductor/m21-jobs-savings-and-billing-ui`,
  the handoff of the Jobs pages. Extended on 2026-11-08 with the short
  resource of a label, from finding 14 of the second review round of that
  branch. A second options check on 2026-11-08 added the rules of F and
  option G.
- **Source:** Implementation.
- **Category:** Mockup.
- **Location:** `docs/mockups/src/screens/jobs/live-notices.tsx:62`,
  `docs/mockups/src/screens/jobs/live-notices.tsx:66`,
  `docs/mockups/src/screens/jobs/live-notices.tsx:82`,
  `docs/mockups/src/screens/jobs/live-notices.tsx:97`,
  `docs/mockups/src/screens/jobs/job-detail.tsx:67`,
  `crates/ciproxy-api/src/wire/jobs.rs:235`,
  `crates/ciproxy-api/src/wire/job_overflow.rs:11`,
  `crates/ciproxy-api/src/wire/job_fit.rs:23`,
  `crates/ciproxy-api/src/handlers/job_fit.rs:78`,
  `crates/ciproxy-protocol/src/resources.rs:34`,
  `crates/ciproxy-agent/src/root_sizing.rs:47`,
  `crates/ciproxy-overflow/src/service/start.rs:173`,
  `crates/ciproxy-overflow/src/service/provider_preparation.rs:56`, and
  `crates/ciproxy-github/src/installation_state.rs:82`.
- **Reason:** D changes the start record, the API, the app, and the
  mockups, so a separate workspace builds it after pull request #53 merges.

- **Terms:**
  - **Known-data scene:** a mockup scene of the fixture story, which shows
    each page with every fact.
  - **Current-contract scene:** a mockup scene that shows the same moment
    with only the facts that the API gives, as under
    `docs/mockups/src/entries/app/jobs/current-contract/` and
    `docs/mockups/src/entries/app/jobs/fit-facts/`.
  - **Largest-size fallback:** the setting "Run on the largest Boat size",
    which runs a job on the largest Boat size when no size fits its label.
  - **Size choice:** the Boat size that the overflow service picks when it
    prepares a sandbox, and its kind: a size that fits the label, or the
    largest-size fallback.
  - **`nothing_fits`:** the API flag of each waiting label in the live
    queue. It is true when no permitted own machine and no permitted Boat
    size fits the label now.
- **Background:**
  - The known-data Jobs scenes name four facts that the API does not give:
    - The person and the day of a suspension: "Tom Becker suspended it on
      GitHub on 19 November", at `live-notices.tsx:97`.
    - The reason of a Boat refusal, with an "Open Boat" action: "The start
      limit of your Boat plan is full until 10:13", at
      `live-notices.tsx:62-66`.
    - The fit of the Boat size that ran a job: "The cheapest Boat size that
      fits the label and the image. Boat bills this size.", at
      `job-detail.tsx:67`.
    - The resource that is short for a label that nothing fits: "No
      machine and no Boat size has 32 vCPU and 128 GB. Change the label, or
      let it run on the largest Boat size.", at `live-notices.tsx:82-83`.
      The label of that scene asks for 32 vCPU, 128 GB, and 200 GB of free
      disk. The scene prints the label through `sizePhrase`, which always
      prints vCPU and memory and never disk, at
      `docs/mockups/src/fixtures/sizes.ts:45-47`. So the scene names a
      short resource only in this story, and it gives no rule for a cause.
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
  - Short resource: `WaitingLabelFit` at
    `crates/ciproxy-api/src/wire/job_fit.rs:11-28` gives `nothing_fits`,
    the Boat fit of the label, and the label with counts of its own
    machines. The Boat fit is `disabled`, `no_size_fits`, `fitting` with a
    size, `largest_fallback` with a size, or `unavailable`. The counts are
    `eligible_machines`, `unavailable_machines`, and `fitting_machines`, at
    `crates/ciproxy-api/src/wire/settings.rs:151-164`. No field names a
    short vCPU, memory, or disk.
  - `crates/ciproxy-api/src/handlers/job_fit.rs:78-90` sets `nothing_fits`
    when no own machine fits, the own fit is known, and the Boat fit is
    `no_size_fits` or `disabled`. So the flag is also true for a label that
    forbids own machines, Boat, or both, and while no own machine takes
    jobs. The own fit and the Boat fit both count free disk, at
    `crates/ciproxy-protocol/src/resources.rs:108-119` and
    `crates/ciproxy-scaler/src/size_policy.rs:120-124`.
  - Own machines have no separate vCPU and memory. A slot is 2 vCPU and
    8 GB, at `docs/protocol/job-routing.md:9`. A job takes the larger of
    its vCPU divided by 2 and its memory divided by 8 GB, at `:25-28`. A
    machine has the smaller of its CPU and its memory slot count after the
    host reserve, at `:34-36`, as
    `crates/ciproxy-protocol/src/resources.rs:34-48` computes. A size fits
    a machine when the machine has those slots and its job disk covers the
    complete VM root, at `:95-96`. So a machine with 32 threads and 512 GB
    has 12 slots, and it takes no job of 2 vCPU and 128 GB, which takes 16
    slots. "No machine has 128 GB" would be false there.
  - One cause across all places has gaps. Boat sizes of 32 vCPU with 64 GB
    and of 16 vCPU with 128 GB each cover one resource of a label of 32 vCPU
    and 128 GB, and neither fits the label. No resource is then short. When
    no permitted place takes jobs, as for a label that forbids Boat while no
    own machine is connected, every resource is short. In the known-data
    story, an AX42 covers 200 GB of free disk and the largest Boat size does
    not, so one set over all places hides the disk that Boat lacks.
  - The product shows only the facts of the API: "The GitHub App is
    suspended on `<organization>`" with "Open GitHub", "Boat refused new
    sandboxes" with the retry time and no action, "Boat refused the size of
    the label `<name>`" for one label, and "Boat bills this size." For a
    label that nothing fits, it names the full request with "fits" and each
    place that the label forbids or that takes no jobs, as in "No machine
    of yours and no Boat size fits a job of 32 vCPU, 128 GB of memory, and
    200 GB of free disk." It never names a short resource. This copy is in
    `ts/app/src/view/jobs-notices.ts`,
    `ts/app/src/view/jobs-boat-notices.ts`,
    `ts/app/src/view/jobs-label-notices.ts`, and
    `ts/app/src/view/jobs-detail.ts`, and the current-contract scenes show
    it. Each sentence is true.
- **Problem:** The known-data scenes and the product disagree on these four
  facts.
- **Impact if no action:** The mockups keep two versions of these notices,
  and a person cannot see who suspended the App, why Boat refused, that a
  job ran on a smaller size than its label, or which resource of a waiting
  label is short.
- **Root question:** What must the Jobs pages tell a person about a
  suspension, a Boat refusal, the Boat size that ran a job, and a waiting
  label that nothing fits?
- **Options:**
  - **A.** Accept the current-contract copy as the spec, and change the
    known-data scenes to it. Effort: Small.
  - **B.** Add the first three facts to the API. The installation lookup reads
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
    copy for the other facts. Risk: the start reason is the intent of the
    scaler, and the size choice at the preparation can differ from it, so
    the saved reason can disagree with the size that ran. Effort: Medium.
  - **D.** Keep the kind of the size choice with each Boat start. When the
    overflow service picks the size at the preparation, it saves whether
    the size fits the label or is the largest-size fallback, next to the
    chosen size in the start record. The job view gives the kind to the
    job page, which words the caption of the Boat size from it. Accept the
    current-contract copy for the suspension, the refusal, and the short
    resource. This drops the assumption of C that the fit comes from the
    start reason of the scaler, because the size that ran comes from the
    later size choice.
    Cost: a field in the stored start record and on the wire. Older starts
    have no value, so they keep "Boat bills this size.". Effort: Medium.
  - **E.** Do D. Also keep the typed reason of a temporary Boat refusal
    next to `retry_at`, and word it in the live notice, such as a full
    start limit of the Boat plan, or no free Boat capacity. Show no "Open
    Boat" action, and leave out the person and the day of a suspension.
    This drops the shared assumption that the facts go together:
    each fact has its own source, cost, and risk. Cost: a field in the
    stored overflow state, a wire field, and copy for five reasons.
    Effort: Large.
  - **F.** Add the short resource to the API. For each waiting label that
    nothing fits, the API names the requested resources that the places do
    not cover: vCPU, memory, or free disk. The fit rules of the scaler give
    the cause, because they count slots, the image root, and the ext4
    allowance, so the app needs no second fit rule. Four rules define the
    cause, and none of them comes from the known-data scene:
    - Take one set for each kind of place: the own machines, and Boat.
    - Count only the places that the label permits and that take jobs now.
      When none remain, give no cause, and name those places as the
      current copy does.
    - Test the own machines against their largest job: the slots of a
      machine times 2 vCPU and 8 GB. The copy then says "takes a job of",
      not "has".
    - Keep the full-request copy when the set of a kind is empty, because
      each resource has a place that covers it but no place covers all of
      them.

    The live notice then names the short resource of each kind. F covers
    the notice of a label with `nothing_fits`. The notice of the
    largest-size fallback keeps its full request. Accept the
    current-contract copy for the other three facts. F adds behaviour at
    the boundary, so it needs a protocol line, a wire field, generated
    types, the app, and the mockups. Risk: the cause names what must
    change, not what fits, so a label can still fit nothing after a person
    lowers the named resources. F does not depend on another option, so
    the user can take it alone or with B, C, D, or E. Taken with C, D, or
    E, F replaces their acceptance of the current-contract copy for the
    short resource. Effort: Medium.
  - **G.** Add the offer of each place to the API. For each waiting label
    that nothing fits, the API names the largest job that one place of
    each permitted kind takes:
    - Own machines count only when one takes jobs now. Their offer is the
      machine with the most slots: its slots times 2 vCPU and 8 GB, and the
      free disk that its job disk holds under its image.
    - The Boat offer is the largest catalog size, which the fallback
      already uses, with the free disk that it holds under the image.

    The live notice shows these offers beside the full request, as in
    "Your largest machine takes jobs of up to 12 vCPU and 48 GB. The
    largest Boat size has 16 vCPU and 32 GB." Each number is a fact of one
    place, so G needs no rule across places. A label within one offer fits
    that place. The live queue already names the offer of the largest Boat
    size when it starts jobs there, at
    `ts/app/src/view/jobs-label-notices.ts:105-114`. The same API facts can
    later feed the fit note of the label dialog. G drops the shared
    assumption that every option describes the request of the label and
    never what the places offer. `docs/protocol/labels.md:127-128` asks
    only where the jobs of each label run now. Accept the current-contract
    copy for the other three facts. G does not depend on another option,
    so the user can take it alone or with B, C, D, or E. Taken with C, D,
    or E, G replaces their acceptance of the current-contract copy for the
    fourth fact. G replaces F for the fourth fact, and G with F names both
    the cause and the offer. Cost: a protocol line, wire fields, generated
    types, the app copy, and the Jobs mockups. Risk: the machine with the
    most slots can have less job disk than another machine. Then the notice
    leaves out a machine that fits a label with fewer vCPU and more disk,
    but each sentence stays true. Risk: the free disk of an offer inverts
    the root rule at `crates/ciproxy-agent/src/root_sizing.rs:47-52`, so
    the backend must round it down and check it with the forward rule.
    Effort: Medium.
- **Left out:**
  - A fit that the page computes from the retained request and the actual
    size, because the page has neither the image root nor the Boat sizes of
    that moment. So it cannot tell the cheapest fit from the fallback in
    every case.
  - A short resource that the page computes from the request and the
    resource offers of each place, because a second fit rule in the app can
    disagree with the scaler, which counts slots, the image root, and the
    ext4 allowance.
  - A refusal at save of a label that nothing fits, because
    `docs/protocol/labels.md:129-132` rejects it, and that decision stays.
  - A list of every place with its offer or its short resources, because
    it is a detail of F or G with a longer notice, not another approach.
- **Wider fix:** None. Each fact needs its own source. The current-contract
  pattern of `docs/mockups/README.md:89-98` and the real-data rule already
  cover the class.
- **Previous examples:** Until 8 November 2026, the product notice of a
  label that nothing fits claimed a short vCPU and memory from
  `nothing_fits` alone. The current copy names the full request instead.
  On 8 November 2026, the user kept the product words of the billing
  banners and dropped the offers of the places from the label dialog.
- **Related:**
  - `plans/follow-ups/job-routing-reason-writer.md` A saves each placement
    fact where it becomes known, also the kind of the size choice at the
    preparation. D of this item saves the same fact, so both changes
    should share one stored field. Neither item waits for the other.
  - `plans/follow-ups/job-cache-cause.md` adds the cause of a direct GitHub
    cache use as another fact of this item when its C lands, with its B as
    part of the API option. This decision does not wait for it.
  - `plans/follow-ups/label-boat-fit-boundary.md` adds the required Boat
    fit of saved and draft labels. A Boat fit is no offer, so keep its
    field apart from any offer field. Neither item waits for the other.
- **Recommendation:** D. Only the fallback changes what a person
  understands about a job that ran, and the size choice at the preparation
  is the true source of that fact. For a waiting label, the current-contract
  copy names the full request and the forbidden places, which an Admin can
  change with Edit label. When the user also wants the fourth fact, G is
  the safer add-on than F, because each number of G is a fact of one place,
  and F first needs its four rules. The first pass recommended C.
- **User answer, 2026-11-08:** D. Rejected: A, B, C, E, F, and G.
- **Action:** fix after merge, apply D, because it is backend work for a
  separate workspace, outside the UI branch.
