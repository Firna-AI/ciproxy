# Label responses lack current Boat start holds

- **Severity:** Medium. **Likelihood:** Common. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-08, from `conductor/m21-jobs-savings-and-billing-ui`,
  a problem outside the scope that the docs merge of the second options
  check of `plans/follow-ups/jobs-notice-facts.md` found.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-api/src/wire/settings.rs`, `LabelFit`,
  `crates/ciproxy-api/src/handlers/label_boat.rs`, `enrich`,
  `crates/ciproxy-api/src/wire/job_overflow.rs`, `JobOverflow`,
  `docs/protocol/labels.md`, "A size that nothing can run", and
  `docs/protocol/labels-order.md`, "Changes to the order".
- **Reason:** Saved and draft labels have physical Boat fit. Their current
  start holds still need the shared server status that
  `plans/follow-ups/machines-boat-start-status.md` records.

- **Terms:**
  - **Placement:** where the jobs of a label run now: the places of its
    order that can run them, the largest size of a provider, or a wait.
    Today the code gives own machines, Boat, or wait.
  - **`LabelFit`:** physical machine and Boat fit, with account presence,
    required root disk, the largest Boat size, and placement.
  - **`BoatLabelFit`:** physical Boat fit shared by Settings and Jobs:
    `unavailable`, `disabled`, `no_size_fits`, `fitting`, or
    `largest_fallback`. A selected size also has a `refused` flag.
  - **Draft label:** a label in the label dialog before the save, which
    the preview reads.
  - **Offer:** the largest job that one place takes. On 8 November 2026,
    the user left offers out of the label dialog and of the notice of a
    waiting label that nothing fits. An offer is not a placement.
- **Background:**
  - On 23 November 2026, the user chose several overflow providers for a
    workspace and an order of places for each label. A place is own
    machines or one provider. The order replaced the label switches "Your
    machines" and "Boat".
  - `docs/protocol/labels.md`, "A size that nothing can run", lines
    216-218, says that the dialog, the label list, and the live queue show
    where the jobs of each label run now: the places of its order that can
    run them, the largest size of a provider, or a wait.
  - `docs/protocol/labels-order.md`, "Changes to the order", lines 51-53,
    says that while a size fits no own machine, jobs skip own machines. The
    dialog says so before the save.
  - At lines 201-211 of `labels.md`, the setting "When nothing fits"
    chooses between waiting and the largest size of the first provider of
    the order that jobs skip only because of its sizes. At lines 226-229, a
    label that nothing fits is never refused at save.
  - At lines 219-225 of `labels.md`, the label list and the Jobs notice
    give the reason of the first place of the order that fits the size,
    such as the spend limit of that provider. So the label list needs the
    current start holds too.
  - These responses carry a saved label with the full `LabelFit`:
    - `GET /api/workspaces/<id>/labels`, at
      `crates/ciproxy-api/src/handlers/labels.rs`, `list`.
    - The replies of `POST /api/workspaces/<id>/labels` and
      `PUT /api/workspaces/<id>/labels/<label id>`, at `create` and
      `update` in the same file.
    - `SettingsView` and `SetupView`, at
      `crates/ciproxy-api/src/wire/settings.rs`.
  - `LabelView` holds the label and `fit: LabelFit` in `settings.rs`.
    `crates/ciproxy-api/src/handlers/label_facts.rs` adds the Boat fit to
    saved labels and draft previews through `label_boat::enrich`.
  - The preview `POST /api/workspaces/<id>/labels/preview` returns the
    draft's full `LabelFit`. It uses the machines, trusted image sizing,
    and existing overflow observation.
  - `WaitingLabelFit` in `crates/ciproxy-api/src/wire/job_fit.rs` gives
    `current.fit`, `boat: BoatLabelFit`, and `nothing_fits`.
    `crates/ciproxy-api/src/handlers/job_fit.rs` enriches `current.fit`
    through the same `label_boat::enrich` call when root sizing is known.
    Its outer Boat choice and `nothing_fits` use `current.fit.boat`.
  - `label_boat::enrich` reads `OverflowView::resource_catalog` directly.
    It uses the scaler size policy and the trusted root sizing.
    Temporary refusals and missing spend bounds do not change physical
    fit. A selected size keeps its refusal flag. Unknown own-machine
    capacity prevents a largest-size fallback.
  - The live page also sends the current Boat facts as `JobOverflow`:
    `connected`,
    `key_rejected`, `billing_required`, `retry_at`, the spend limit, and
    `refused_sizes`, at `crates/ciproxy-api/src/wire/job_overflow.rs`.
    The app words a current Boat hold from them.
  - Saved labels and drafts now give physical placement even without
    waiting jobs. They still give no current start status for a rejected
    key, billing, a retry time, or an accounting hold.
  - The current clients:
    - Settings calls the preview, create, and update routes through
      `ts/app/src/api/settings-endpoints.ts`.
    - `ts/app/src/view/settings-labels.ts` reads the server placement.
      `ts/app/src/view/settings-label-fit.ts` shows the full physical fit
      in the label dialog.
    - Setup and Settings read the shared fit through
      `ts/app/src/api/parse-setup.ts`. The live queue reads `current.fit`
      and the outer Boat fit through `ts/app/src/api/jobs-live-parse.ts`.
    - `npm --prefix ts run api:generate` regenerates the types in
      `ts/api/generated/`.
  - `crates/ciproxy-api/tests/http/m20_label_tests.rs` covers saved and
    draft Boat fit, refusals, fallback, and unknown facts.
    `crates/ciproxy-api/tests/http/m18_fit_tests.rs` covers the matching
    nested and outer Boat choices for waiting labels.
  - On 17 November 2026, the user chose one physical Boat fit calculation
    for Settings and Jobs. That work does not add current start status.
- **Problem:** Physical Boat fit does not tell a client whether Boat can
  start a new job now. Saved and draft label responses lack the current
  key, billing, retry, spend limit, and accounting holds.
- **Impact if no action:** Settings can name Boat as the physical placement
  while new starts wait. The label list and dialog cannot explain that hold.
- **Root question:** How do saved and draft labels learn whether Boat can
  start their jobs now?
- **Options:**
  - **A.** Add current Boat start status to the existing saved and draft
    label responses:
    - Keep the shared physical `LabelFit` computation.
    - Give these responses the current Boat status once. Reuse the server
      status of `plans/follow-ups/machines-boat-start-status.md` so that
      Machines, Jobs, and Settings share one hold rule.
    - Keep the physical Boat choice separate from current start status.
      A refused fitting size stays visible as a physical fit.
    - Add no validation, no refusal at save, and no other behaviour. The
      protocol keeps its lines, and the API types are regenerated.
    - Extend the existing HTTP fit tests with the missing hold case.
      Keep the physical-fit assertions and the unknown-observation cases.
    - Effort: Medium.
  - **B.** Add a separate read for the current Boat start status of saved
    labels. Add the same status to the preview. It uses the same server
    rule and regression case as A. Saved-label types stay unchanged, but
    Settings needs a second request and the API gains a route.
    Effort: Medium.
- **Left out:**
  - A Boat fit that the app computes from a Boat catalog, because the API
    already gives the shared physical fit and placement.
  - The Boat fit of the live queue for saved labels, because it covers only
    labels with waiting jobs, and never a draft.
  - "Boat" for every label whose order holds Boat and that fits no own
    machine, because a label that no Boat size fits then shows Boat while
    its jobs wait.
  - A protocol change that drops the places and the wait from the label
    list and the dialog, because `docs/protocol/labels.md`, "A size that
    nothing can run", lines 216-218, stays in force.
  - The offers of the places, because the user left them out on
    8 November 2026, and they do not identify current start holds.
  - A refusal at save of a label that nothing fits, because the decision at
    lines 226-229 of `docs/protocol/labels.md` rejects it.
- **Wider fix:** A. One server start status serves the live queue, the
  label list, and the dialog. Clients then need no hold rule of their own.
- **Previous examples:** Settings and Jobs now use `label_boat::enrich`
  for physical fit. The separate current-start gap also affects Machines,
  as `plans/follow-ups/machines-boat-start-status.md` records.
- **Related:**
  - On 8 November 2026, the user kept the label dialog on its fit counts
    and placement, and its mockup dropped the offers of the places. This
    physical fit is now present. A physical fit, an offer, and current
    start status describe different facts.
  - `plans/follow-ups/machines-boat-start-status.md` owns the shared server
    status. Reuse its source when adding status to label responses.
  - `plans/follow-ups/jobs-notice-facts.md` keeps the kind of the Boat size
    choice of each start. On 8 November 2026, the user chose that option,
    not the offers of its option G. Neither item waits for the other.
- **Recommendation:** A. Existing responses then give current start status
  from one server rule, with no second request or copied client rule.
  B keeps saved-label types unchanged but adds a request and a route.
- **Action:** fix after merge, apply A, because the remaining shared
  start-status work reaches the overflow, scaler, API, and app code.
