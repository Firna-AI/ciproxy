# Cached Setup progress stays open off Setup

- **Severity:** Low. **Likelihood:** Common. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-07, from `conductor/mvp-m19-ui`, PR #36, a new problem
  that the finding check of review finding 6 reported after the last fix
  round of that review.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `ts/app/src/screens/workspace-page.tsx:35`,
  `useSetupProgress`, `ts/app/src/screens/landing.tsx:42`, in `open`,
  `ts/app/src/screens/coming-soon.tsx:19`, `SetupAction`, and
  `ts/app/src/state/setup.ts:94`, `watch`.
- **Reason:** The review found this after its last fix round, and the stale
  view costs at most one extra Setup visit.

- **Terms:**
  - **Setup:** the onboarding page of an Admin at `/setup?workspace=<id>`,
    with three steps: the GitHub App, a machine, and the first job.
  - **Setup store:** `createSetupStore` in `ts/app/src/state/setup.ts`. The
    app keeps one for each workspace for the whole document. The shell of an
    Admin, Continue setup, the landing, and the Setup page read it.
  - **Shell progress:** the "Set up" entry with "N of 3" that the navigation
    shows first while Setup is open. `navItems` in
    `ts/app/src/view/shell.ts` builds it.
  - **Workspace landing:** the page `/?workspace=<id>`. It opens Setup for
    an Admin while Setup waits, else Jobs.
- **Background:**
  - `useSetupProgress` gives the shell progress of an Admin. It calls
    `ensure` of the Setup store, which reads Setup once for each document.
  - `SetupAction` shows "Continue setup" on the pages that later work
    fills, while the stored Setup is not complete. It reads the same store.
  - Only the Setup page polls. It calls `load` and `watch` on mount, at
    `ts/app/src/screens/setup/setup-screen.tsx:31`. Leaving Setup stops the
    poll.
  - The workspace switch opens the workspace landing, at
    `ts/app/src/view/shell.ts:98`.
  - `open` in `landing.tsx` calls `ensure`. Then it decides from the stored
    state, with `workspaceLanding` of `ts/app/src/routing/landing.ts`.
  - `ts/app/README.md` says that `/?workspace=<id>` opens Setup for an
    Admin until it is done, else Jobs.
  - `watch` returns a stop function that stops the one poller of the store
    for every caller. So a second watcher would lose its poll when the
    first one stops.
  - The poller stops by itself when Setup no longer waits: a machine
    joined, and the first job completed.
  - The browser smoke harness can change its fixture scenario during a
    case, with `control` in `ts/app/scripts/browser-fixture.ts` and
    `reset: false`. The scenarios `setup-started` and `setup-success`
    exist.
- **Problem:** Leave Setup while the first job runs, and let that job
  finish. No read runs off Setup, so the shell progress and Continue setup
  stay open. A workspace switch then uses the stored running state and
  opens Setup instead of Jobs, where a new read shows the completion.
- **Impact if no action:** The Admin sees stale onboarding progress, or
  opens Setup once more than needed. A reload or a Setup visit corrects it.
- **Root question:** How should workspace landing and shell progress stop
  showing completed onboarding as open?
- **Options:**
  - **A.** Read Setup again before each workspace landing decision. It
    removes the extra Setup visit, but the shell progress stays stale while
    the current page stays open. Effort: Small.
  - **B.** Watch Setup from the shared shell hook while it is incomplete,
    and read Setup again for each workspace landing decision. Share the one
    poller through the existing store, and stop it at completion. Effort:
    Small.
- **Left out:**
  - Poll after completion: it adds requests with no need.
  - Keep the stored state: it keeps the stale progress.
- **Wider fix:** B. One hook gives every shell consumer current progress
  while Setup is incomplete.
- **Previous examples:** The session store reads again on each page change
  through `visit`, so that an ended role shows on the next page. The Setup
  page already watches its store while it shows.
- **Related:** `plans/follow-ups/explicit-session-refresh-order.md` changes
  the same Setup store. Its recommended helper decides which reads share an
  open read, and which reads start a new one. The shell watch of B is a
  background read, and the landing read is an explicit read. Neither fix
  waits for the other. The fix that lands second follows the rule of the
  first.
- **Recommendation:** B. One shared poller keeps every shell consumer
  current, and it stops at completion.
- **Action:** fix after merge, apply B, because the review found it after
  its last fix round, and a reload or a Setup visit corrects the view.
  - Before the change, run the browser case below on the current code, and
    confirm that it fails.
  - Count the watchers in the Setup store. `watch` starts the poller for
    the first watcher. Its stop function stops the poller only after the
    last watcher stops.
  - In `useSetupProgress`, keep `ensure` for the first read. Then watch the
    store while the stored Setup still waits, and stop on unmount.
  - In `open` of `landing.tsx`, read Setup again before `workspaceLanding`
    decides.
  - Keep the Setup page as it is. Its `load` and `watch` share the same
    poller.
  - Add a Node test with `appWith` and `manualSchedule` from
    `ts/app/_tests_/support.ts`. Two watchers share one poller. A stop of
    one watcher keeps the poll, and a stop of both ends it. Completion ends
    it for both.
  - Add a browser smoke case. Open Setup with the scenario `setup-started`,
    then open Jobs. Change the scenario to `setup-success` with
    `reset: false`. Wait until the "Set up" entry and Continue setup are
    gone.
  - Then open the same workspace from the workspace switch, and expect Jobs.
    Run the case at 1440, 720, and 390 pixels.
  - Run `node --test ts/app/_tests_/setup.test.ts`, and run the smoke as
    `ts/app/README.md` says. Run each new test 10 times.
