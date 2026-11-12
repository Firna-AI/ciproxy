# Dashboard scripts

The scripts of `@ciproxy/app` that check the dashboard in a browser. The
fixture harness serves the static export with a typed fixture API, and the
browser smokes drive the pages against it. The live checks read a deployed
dashboard with a real sign-in. All of them are Node scripts in TypeScript,
with no extra runner. The [dashboard guide](../README.md) describes the pages
that they check.

Run each command from the repository root, with the Node version in
[.nvmrc](../../../.nvmrc). The repository has no browser dependency, so a
browser script needs `playwright-core` from outside the repository.

## Fixture harness

`harness-server.ts` serves the export in `ts/app/dist` and a typed fixture
API on `127.0.0.1`. It contacts no live API, makes no live payment request,
and runs until Ctrl+C.

```sh
npm --prefix ts run app:export
node ts/app/scripts/harness-server.ts --port=4340
```

- `--port` sets the port, by default 4340. The harness prints its origin.
- The fixture API answers the dashboard routes with responses of the
  generated types. It serves the sign-in and Setup pages, and the Jobs,
  Savings, and billing pages for each role.
- `POST /__fixture/control` with a JSON body changes the state:
  - `scenario` names a set of responses. A new scenario also clears the
    recorded requests and counters, unless `reset` is `false`.
  - `pages` changes the Jobs, Savings, and billing responses of the current
    scenario, such as the role, a failed or held route, or the next payment
    outcome. With `reviewRequired`, each paid change gets
    `409 review_required`, as for a quote that the harness did not give.
  - `build` sets the API build that the responses name, for the reload after
    a new build.
  - An invalid body gets status 400 and changes nothing.
- `GET /__fixture/state` gives the scenario, the build, the install commands
  that the harness issued and revoked, and the requests that it saw.
- Each other path under `/__fixture/` is a stand-in page for GitHub or
  Stripe, such as the App install, a Stripe Checkout, the Stripe portal, or a
  bank confirmation. No real payment or GitHub call happens.
- Start the harness again to load a changed fixture module.

## Browser checks

The browser smokes drive headless Chrome against the harness at 1440, 720,
and 390 pixels. Start the harness first, then run a smoke in a second
terminal:

```sh
export CIPROXY_PLAYWRIGHT_PATH=/absolute/path/playwright-core/index.mjs
node ts/app/scripts/browser-smoke.ts
node ts/app/scripts/browser-pages-smoke.ts --list
node ts/app/scripts/browser-pages-smoke.ts --case=NAME --repeat=10
```

The shell smoke, `browser-smoke.ts`, covers the sign-in and Setup pages and
their states, the shell, and the reload after a build change.

The page smoke, `browser-pages-smoke.ts`, covers the states of Jobs, the job
page, Savings, Plan and usage, Plans, the Checkout review and return, and the
banner. It also covers each page while it loads, and Viewer and Billing views
of Jobs, Savings, and the billing pages. Its journeys follow the History
filters to a job and back, change the Savings range, recover a failed read,
and take an offer through review, payment, and return. Other journeys cover a
paid change with a bank confirmation, a refused quote that needs a new review
and a new click, and a return that names its workspace.
A Viewer case checks that a saved Stripe address gives a Viewer no payment
action.

- `--case=NAME` runs one case or a comma-separated list of cases. A name
  that matches no case fails the run. With no `--case`, a smoke runs every
  case.
- `--list` prints the cases of the page smoke. It exits with status 2 when
  no case exists.
- `--repeat=COUNT` runs each selected page case `COUNT` times, each run in a
  new browser context and in its own `run-NN` directory. A failed run stays a
  failure, and no case runs again by itself.
- `--width=WIDTH` runs the page smoke at one width or a comma-separated list
  of widths: 1440, 720, or 390.
- `--stop-on-failure` ends the page smoke at its first failed case.
- `CIPROXY_PLAYWRIGHT_PATH` names `playwright-core/index.mjs`. A browser
  smoke stops without it.
- `CIPROXY_CHROME_PATH` sets the browser, by default `/usr/bin/google-chrome`.
- `CIPROXY_HARNESS_URL` sets the harness, by default `http://127.0.0.1:4340`.
- `CIPROXY_SMOKE_OUTPUT` sets the directory for the screenshots,
  `results.json`, and a trace of each failed case. By default, it is
  `.context/ciproxy-mvp-m19/browser-smoke` for the shell smoke and
  `.context/m21/browser-smoke` for the page smoke, under the current
  directory.

### Repeated runs

- The root `results.json` of the page smoke records every case, width,
  attempt, and duration.
- Each attempt runs every assertion, including the viewport and panel
  bounds. The runner adds no retry and keeps every failed result.
- Only the first attempt saves the screenshots of successful checks, also
  those that a journey takes, and records a trace. A later successful
  attempt saves neither.
- A failed attempt always saves a failure screenshot. Only the first attempt
  also saves a failure trace, so run a failed case once more for its full
  trace.
- The choice of evidence changes no assertion, matcher, or timeout.
- Parallel runs need separate harness state and separate output paths.

## Live checks

Both live checks use the sign-in of a person on a deployed dashboard. They
print only the name and the result of each check, and write no screenshot
and no trace.

The credential file holds a live sign-in. Keep it outside the repository,
and never share its values. It is a JSON object with four strings:

- `base_url`: the HTTPS origin of the dashboard, with no path.
- `workspace_id`: a workspace of the person.
- `session_cookie`: the value of the `__Host-ciproxy` cookie of the sign-in.
- `csrf_token`: the `csrf_token` of `GET /api/session` for the sign-in.

Without the credential variable, each check prints `SKIP` and exits with
status 2.

### Shell live check

The shell check needs an Admin of `workspace_id`. It reads the session,
Setup, and the machines, and checks the build header. It creates one install
command and revokes it at once. Then it reloads the Jobs page at each width.

```sh
CIPROXY_DASHBOARD_CREDENTIALS_FILE=/absolute/path/outside/repo/dashboard.json \
CIPROXY_PLAYWRIGHT_PATH=/absolute/path/playwright-core/index.mjs \
  node ts/app/scripts/live-smoke.ts
```

### Page live check

The page check calls the API only. It takes the path of the credential file
from `CIPROXY_DASHBOARD_CREDENTIALS_FILE` and nothing else from the
environment.

```sh
export CIPROXY_DASHBOARD_CREDENTIALS_FILE=/absolute/path/outside/repo/dashboard.json
node ts/app/scripts/live-pages-smoke.ts --step=read
node ts/app/scripts/live-pages-smoke.ts --step=checkout
node ts/app/scripts/live-pages-smoke.ts --step=return
```

- `--step=read`, the default, works for every role. It reads Jobs, one job
  when the workspace has one, every Savings range, Plan and usage, Plans,
  and the banner, and checks the build header on each response.
- `--step=checkout` needs a Free workspace, a person who may pay, and no
  pending payment. It reviews one choice and starts one Stripe test Checkout
  through the payment module of the app.
- Then complete the Stripe test payment by hand in a browser.
- `--step=return` checks the exact saved Checkout session, then checks that
  the plan matches the reviewed choice after confirmation. A `confirming`
  outcome prints `PENDING Checkout payment confirmation` and exits 2.
  Run it again later. An `open` outcome instead prints
  `PENDING Checkout payment needs completion`. Complete the hosted payment
  first. A busy 409 answer prints
  `PENDING Checkout return reconciliation busy`. Run return again later.
- An `unfinished` outcome prints `FAIL Checkout expired without payment`
  and exits 1. Start a fresh Checkout with a new `result_path`, because the
  existing file reserves the old payment. A `payment_required` return also
  fails: the Checkout return does not produce that paid-change outcome.
  Other HTTP failures remain failures. A 403 does not prove expiry.

The page check also reads three optional fields of the credential file:

- `result_path`: a private JSON file outside the repository, in a folder that
  exists. Checkout and return need it. Checkout creates it only when it does
  not exist yet, so one file holds one payment.
- `selection`: the generated plan choice to review, such as
  `{"tier": "team", "add_ons": {"start_blocks": 0, "machines": 0}}`. Without
  it, Checkout reviews the offer of Plan and usage.
- `runtime_redirects`: the exact `success_url`, `cancel_url`, and optional
  `portal_return_url` of the billing runtime. Checkout needs it. The success
  URL must open `/checkout-return` on `base_url` with
  `session_id={CHECKOUT_SESSION_ID}`. A `workspace` in it must be
  `{WORKSPACE_ID}`, which the runtime fills, or the `workspace_id` of the
  file.

The Stripe address and the session ID go only to the result file, which the
check writes for its owner alone. Never copy them into a log or a message.
The check exits with status 0 when every check passed, 1 when a check or the
file failed, and 2 when the credentials are absent, a check was skipped, or
the payment still confirms, remains open, or reconciliation is busy.

## Key Code

- [harness-server.ts](harness-server.ts) serves the export and routes the
  fixture API, the controls, and the stand-in provider pages.
  [harness-state.ts](harness-state.ts) applies a control to the state.
- [harness-pages-controls.ts](harness-pages-controls.ts) defines the `pages`
  control, and [harness-pages-scenarios.ts](harness-pages-scenarios.ts) the
  scenarios of the Jobs, Savings, and billing pages.
- Each `fixtures-*.ts` module builds the responses of one area with the
  generated types. Many Node tests in `../_tests_` use them too.
- [browser-smoke.ts](browser-smoke.ts) runs the shell smoke, and
  [browser-pages-smoke.ts](browser-pages-smoke.ts) the page smoke, whose
  cases [browser-pages-registry.ts](browser-pages-registry.ts) lists.
  [browser-pages-runtime.ts](browser-pages-runtime.ts) runs each case and
  keeps its evidence.
- [live-smoke.ts](live-smoke.ts) runs the shell live check, and
  [live-pages-smoke.ts](live-pages-smoke.ts) the page live check.
  [live-pages-private.ts](live-pages-private.ts) keeps the credential and
  result files outside the repository.

## Related docs

- The [dashboard guide](../README.md), with its pages and its API rules.
- The [deployment](../../../deploy/README.md#environments), which serves the
  dashboard and `/api` on one origin.
