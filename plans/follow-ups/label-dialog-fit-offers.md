# Label dialog names offers that the API lacks

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-08, from `conductor/m21-jobs-savings-and-billing-ui`,
  a problem outside the scope that the second options check of
  `plans/follow-ups/jobs-notice-facts.md` found.
- **Source:** Review.
- **Category:** Mockup.
- **Location:** `docs/mockups/src/screens/settings/label-fit.tsx:44`,
  `docs/mockups/src/screens/settings/label-fit.tsx:57`,
  `crates/ciproxy-api/src/handlers/labels.rs:44`,
  `crates/ciproxy-api/src/wire/settings.rs:151`, and
  `docs/protocol/labels.md:82`.
- **Reason:** The choice decides which side of a mockup and API mismatch is
  right, and that choice belongs to the user.

- **Terms:**
  - **Label dialog:** the Settings dialog that adds or edits a label: its
    name, its size of vCPU, memory, and free disk, its permitted places,
    and its setting "When nothing fits". The app has no label screen yet.
  - **Fit note:** the line of the label dialog that says, before the save,
    where jobs of the size run. `FitNote` at
    `docs/mockups/src/screens/settings/label-fit.tsx:28-61` draws it.
  - **Offer:** the largest job that one place takes. For an own machine, it
    is its slots times 2 vCPU and 8 GB. For Boat, it is the largest catalog
    size.
  - **Slot:** 2 vCPU and 8 GB of an own machine, as
    `docs/protocol/job-routing.md:9` defines it.
  - **Known-data scene:** a mockup scene of the fixture story, which shows
    each page with every fact.
- **Background:**
  - The label dialog mockup shows `FitNote`, at
    `docs/mockups/src/screens/settings/label-dialog.tsx:174`. The fit note
    has three cases:
    - An own machine fits: "Each of your machines runs 3 jobs of this size
      at a time.", or a range, at `label-fit.tsx:35-42`.
    - No machine fits, and a Boat size does: "No machine fits 16 vCPU and
      32 GB", with "Jobs with this label run on Boat. Your machines offer
      12 vCPU and 48 GB each.", at `label-fit.tsx:44-50`.
    - Nothing fits: "No machine and no Boat size fits 32 vCPU and 128 GB",
      with "Your machines offer 12 vCPU and 48 GB each, and the largest
      Boat size offers 16 vCPU and 32 GB.", at `label-fit.tsx:52-59`.
  - The known-data scenes `no-machine-fits` and `nothing-fits` in
    `docs/mockups/src/entries/app/settings/labels/states.mockup.tsx` show
    these notes. The offers name vCPU and memory only, through
    `sizePhrase`. The label pages have no scene with only the facts of the
    API.
  - The preview `POST /api/workspaces/<id>/labels/preview` takes a draft
    label and returns `LabelFit`, at
    `crates/ciproxy-api/src/handlers/labels.rs:44-65`. `LabelFit` at
    `crates/ciproxy-api/src/wire/settings.rs:151-164` holds counts only:
    `eligible_machines`, `unavailable_machines`, `fitting_machines`,
    `minimum_jobs`, `maximum_jobs`, and `no_machine_fits`. It names no
    offer of a machine and no Boat fact. The saved labels of the settings
    view carry the same `LabelFit`, at `settings.rs:140-147`.
  - `docs/protocol/labels.md` asks the dialog for counts and places:
    - At `:82-85`, how many jobs of the size fit on each own machine at a
      time, with a range when the machines differ.
    - At `:89-92`, how many machines the size fits, such as "1 job on 3 of
      your 4 machines", from the connected machines that can run jobs.
    - At `:105-107`, while the size fits no own machine, the label runs on
      Boat only, and the dialog says so before the save.
    - At `:127-128`, the dialog, the label list, and the live queue show
      where the jobs of each label run now: own machines, Boat, or wait.
    - No line asks for the offers of the places.
  - An offer comes from the fit rules of the backend. A machine has the
    smaller of its CPU and its memory slot count after the host reserve,
    at `docs/protocol/job-routing.md:34-36`. A size also needs a job disk
    that covers the complete VM root, at `:95-96`. The free disk under the
    image inverts the root rule at
    `crates/ciproxy-agent/src/root_sizing.rs:47-52`. So a page that
    computes an offer holds a second fit rule.
  - This item covers only the offers. The choice between the two notices
    above needs the Boat fit of the draft, which the preview does not give
    either. `plans/follow-ups/label-boat-fit-boundary.md` adds it.
- **Problem:** The known-data fit note names the offer of the machines and
  of the largest Boat size. The preview of the API gives counts only, so
  the product cannot show those offers from the facts of the API.
- **Impact if no action:** The mockup and the product keep two versions of
  the fit note, and the label dialog of the app cannot follow the mockup
  without a fit rule of its own. The choice holds only the offers of the
  fit note, not the rest of the label dialog.
- **Root question:** When a size fits no own machine, what must the label
  dialog tell a person about what the places can run?
- **Options:**
  - **A.** Keep the counts and places that the API gives, and change the
    known-data fit note to name only them: where the jobs run, and the full
    request of the size with "fits", as the live queue does. Drop the
    offers from `label-fit.tsx`, and keep the story of each scene. Effort:
    Small.
  - **B.** Get typed offers of each permitted kind of place from the API,
    as option G of `plans/follow-ups/jobs-notice-facts.md` gives them:
    - The own machine with the most slots: its slots times 2 vCPU and 8 GB,
      and the free disk that its job disk holds under its image.
    - The largest Boat size, with the free disk that it holds under the
      image.

    The preview returns them, and the fit note shows them as the
    known-data copy does. The page computes no fit. Cost: a protocol line,
    wire fields, generated types, the app, and the mockups. When the user
    also takes G for the Jobs pages, one API field serves both. Risk: the
    machine with the most slots can have less job disk than another
    machine. Risk: the free disk inverts the root rule, so the backend must
    round it down and check it with the forward rule. Effort: Medium.
- **Left out:**
  - An offer that the page computes from the machine list and the Boat
    catalog, because a second fit rule in the app can disagree with the
    backend, which counts slots, the image root, and the ext4 allowance.
  - No change, because the mockup and the product then keep two versions
    of the fit note.
  - A list of every machine with its offer, because it is a detail of B
    with a longer note.
  - A refusal at save of a size that nothing fits, because
    `docs/protocol/labels.md:129-132` rejects it.
  - Only two approaches are real. Each other approach keeps two versions,
    computes a second fit rule in the app, or is a detail of B.
- **Wider fix:** None. The current-contract pattern of
  `docs/mockups/README.md:89-98` and the real-data rule already cover the
  class.
- **Previous examples:** `plans/follow-ups/jobs-notice-facts.md` records the
  same kind of gap for the Jobs pages, whose known-data notice names a
  short resource that the API lacks.
- **Related:**
  - `plans/follow-ups/jobs-notice-facts.md` G adds the same offers to the
    API for the live queue. When the user takes G there and B here, one API
    field serves both. Neither decision waits for the other.
  - `plans/follow-ups/label-boat-fit-boundary.md` adds the required Boat
    fit of saved and draft labels. The fit note of either option needs it
    to say where the jobs run. A Boat fit is no offer, so the two need
    separate fields. This decision does not wait for it.
- **Recommendation:** A. It keeps the fit note on the counts and places
  that the protocol asks for, and the preview already shows how many jobs
  of each draft size fit before the save. B is worth its cost when the user
  takes G for the Jobs pages, because one API field then serves both.
- **Action:** ask the user, because the choice decides which side of a
  mockup and API mismatch is right.
