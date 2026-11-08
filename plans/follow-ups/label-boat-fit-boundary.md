# Label responses cannot tell Boat from wait

- **Severity:** Medium. **Likelihood:** Common. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-08, from `conductor/m21-jobs-savings-and-billing-ui`,
  a problem outside the scope that the docs merge of the second options
  check of `plans/follow-ups/jobs-notice-facts.md` found.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-api/src/wire/settings.rs:140`,
  `crates/ciproxy-api/src/wire/settings.rs:151`,
  `crates/ciproxy-api/src/handlers/labels.rs:44`,
  `crates/ciproxy-api/src/handlers/job_fit.rs:111`,
  `crates/ciproxy-api/src/handlers/job_fit.rs:133`,
  `crates/ciproxy-api/src/wire/job_fit.rs:33`, and
  `docs/protocol/labels.md:127`.
- **Reason:** The fix is API work outside the UI branch, and only the label
  screens of Settings, which the dashboard has not built yet, depend on it.

- **Terms:**
  - **Placement:** where the jobs of a label run now: own machines, Boat,
    or wait.
  - **`LabelFit`:** the own-machine fit of a label: counts of eligible,
    unavailable, and fitting machines, and the jobs that fit on each.
  - **`BoatLabelFit`:** the Boat fit of a label that the live queue
    already gets: `unavailable`, `disabled`, `no_size_fits`, `fitting` with
    a size, or `largest_fallback` with a size. A fitting size also has a
    `refused` flag.
  - **Draft label:** a label in the label dialog before the save, which
    the preview reads.
  - **Offer:** the largest job that one place takes. On 8 November 2026,
    the user left offers out of the label dialog and of the notice of a
    waiting label that nothing fits. An offer is not a placement.
- **Background:**
  - `docs/protocol/labels.md:127-128` says that the dialog, the label list,
    and the live queue show where the jobs of each label run now: own
    machines, Boat, or wait. At `:105-107`, while a size fits no own
    machine, the label runs on Boat only, and the dialog says so before the
    save. At `:118-126`, the setting "When nothing fits" chooses between
    waiting and the largest Boat size. At `:129-132`, a label that nothing
    fits is never refused at save.
  - These responses carry a saved label, and each gives only `LabelFit`:
    - `GET /api/workspaces/<id>/labels`, at
      `crates/ciproxy-api/src/handlers/labels.rs:31-42`.
    - The replies of `POST /api/workspaces/<id>/labels` and
      `PUT /api/workspaces/<id>/labels/<label id>`, at `labels.rs:68` and
      `labels.rs:102`.
    - `SettingsView` and `SetupView`, at
      `crates/ciproxy-api/src/wire/settings.rs:28-34` and `:11-23`.
  - `LabelView` at `settings.rs:140-147` holds the label and
    `fit: LabelFit`. `LabelFit` at `settings.rs:151-164` holds the
    own-machine counts only.
  - The preview `POST /api/workspaces/<id>/labels/preview` reads a draft
    label and the machines only, and returns `LabelFit`, at
    `labels.rs:44-65`.
  - The live queue already gets the Boat fit, but only for labels with
    waiting jobs. `WaitingLabelFit` at
    `crates/ciproxy-api/src/wire/job_fit.rs:11-28` gives
    `boat: BoatLabelFit`, at `job_fit.rs:33-58`, and `nothing_fits`.
    `crates/ciproxy-api/src/handlers/job_fit.rs` computes them:
    - `physical_catalog` at `:111-131` takes the Boat catalog from the
      overflow view, without its temporary refusals and spend.
    - `boat_fit` at `:133-178` applies the root sizing of the agent, the
      size policy of the scaler, and the setting "When nothing fits".
    - The `nothing_fits` rule is at `:78-90`.
  - The live page also sends the current Boat facts as `JobOverflow`, at
    `crates/ciproxy-api/src/wire/jobs.rs:213`: `connected`,
    `key_rejected`, `billing_required`, `retry_at`, the spend limit, and
    `refused_sizes`, at `crates/ciproxy-api/src/wire/job_overflow.rs:11-32`.
    The app words a current Boat hold from them.
  - So a client can tell from the saved and draft responses whether an own
    machine fits, but not whether Boat runs the jobs or they wait. A label
    with no waiting job, and every draft, has no Boat fit in any response.
  - The current clients:
    - No app code calls the label list, the preview, or the create and
      update routes. The label screens of Settings are not built.
    - The Setup parser reads the `LabelFit` of the Setup labels field by
      field, at `ts/app/src/api/parse-setup.ts:72-81`. The live queue reads
      the own fit from `current.fit`, at
      `ts/app/src/view/jobs-places.ts:112-113`. Both ignore an added field.
    - `npm --prefix ts run api:generate` regenerates the types in
      `ts/api/generated/`.
    - So the fix costs the same while it waits. It gets harder only if the
      label screens of Settings are built before it, with a placement of
      their own.
  - The label list and the label dialog of Settings depend on this item for
    their placement. The Jobs, Savings, and billing pages do not, because
    the live queue already gets the Boat fit of each waiting label.
  - The existing API tests: `crates/ciproxy-api/tests/http/settings_tests.rs:16`
    covers the label list and the preview, with own fit only, at `:27` and
    `:48`. `crates/ciproxy-api/tests/http/m18_fit_tests.rs:24` covers the
    Boat fit states of waiting labels.
- **Problem:** The label list and the label dialog must show whether the
  jobs of a label run on own machines, on Boat, or wait. The saved and
  draft responses give only the own-machine fit, so a client cannot tell
  Boat from wait without a fit rule of its own.
- **Impact if no action:** The label screens of Settings cannot meet
  `docs/protocol/labels.md:127-128`. They must leave out the placement of
  each label that no own machine fits, or guess it.
- **Root question:** How do the label list and the label dialog learn where
  the jobs of a saved or a draft label run now?
- **Options:**
  - **A.** Add to the saved and draft responses the Boat facts that the API
    already computes for the live queue:
    - Give each saved label of the label list, the Settings view, and the
      create and update replies, and the draft of the preview, its
      `BoatLabelFit` and its `nothing_fits` result. Compute them with
      `physical_catalog`, `boat_fit`, and the `nothing_fits` rule of
      `crates/ciproxy-api/src/handlers/job_fit.rs`, from the same label,
      machines, root sizing, catalog, and overflow view. Move that code to
      one place that every route calls.
    - Give these responses the current Boat facts once, as the live page
      gives them in `JobOverflow`, so that one hold rule serves the live
      queue and the label screens.
    - Where the shared `LabelView` also appears in the live queue, both
      Boat fits come from the same call, so they cannot differ.
    - Add no validation, no refusal at save, and no other behaviour. The
      protocol keeps its lines, and the API types are regenerated.
    - Add tests at the API boundary, beside the test at
      `settings_tests.rs:16`. For the label list and for the preview, add
      one case for each placement:
      - An own machine fits.
      - No own machine fits, and a Boat size fits.
      - The label forbids Boat.
      - No Boat size fits, and the label waits.
      - The label uses the largest-size fallback.
      - Boat facts are missing, which gives `unavailable`.
      - Boat refused the fitting size.
    - Also test that a saved label with waiting jobs gives the same Boat fit
      in the label list and in the live queue, and that a label that
      nothing fits still saves.
    - Effort: Medium.
  - **B.** Add a separate read that returns the placement of each saved
    label, and add the same Boat facts to the preview. It uses the same
    computation and the same tests as A. The saved-label types stay as they
    are, but the label screens need a second request, and the API gets a
    new route. Effort: Medium.
- **Left out:**
  - A Boat fit that the app computes from a Boat catalog, because no label
    response sends a catalog, and a second fit rule in the app can disagree
    with the scaler, which counts the image root and the ext4 allowance.
  - The Boat fit of the live queue for saved labels, because it covers only
    labels with waiting jobs, and never a draft.
  - "Boat" for every label that allows Boat and fits no own machine,
    because a label that no Boat size fits then shows Boat while its jobs
    wait.
  - A protocol change that drops Boat and wait from the label list and the
    dialog, because `docs/protocol/labels.md:127-128` stays in force.
  - The offers of the places, because they are an optional extra that the
    user left out on 8 November 2026, and they do not say where the jobs
    run.
  - A refusal at save of a label that nothing fits, because
    `docs/protocol/labels.md:129-132` rejects it.
- **Wider fix:** A. One owner computation of the Boat fit then serves the
  live queue, the label list, and the dialog, so no client needs a fit rule
  of its own.
- **Previous examples:** The live queue gained the Boat fit of each waiting
  label, at `crates/ciproxy-api/src/wire/job_fit.rs:33-58`, but the saved
  and draft responses did not.
- **Related:**
  - On 8 November 2026, the user kept the label dialog on its fit counts
    and placement, and its mockup dropped the offers of the places. This
    item stays required, because the dialog still needs the Boat fit to
    show its placement. The Boat fit says whether Boat runs the jobs, and an
    offer says the largest job that a place takes, so the two need separate
    fields.
  - `plans/follow-ups/jobs-notice-facts.md` keeps the kind of the Boat size
    choice of each start. On 8 November 2026, the user chose that option,
    not the offers of its option G. Neither item waits for the other.
- **Recommendation:** A. One response then gives each label its placement
  from the computation that the live queue already uses, with no second
  request and no rule in the app. B keeps the saved-label types as they
  are, but it adds a request and a route.
- **Action:** fix after merge, apply A, because the change is API work
  outside the UI branch, and only the label screens of Settings, which the
  dashboard has not built yet, depend on it.
