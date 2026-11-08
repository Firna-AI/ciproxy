# @ciproxy/app

The private ciproxy dashboard, built with Expo Router and React Native Web.
It targets web only and has no open licence.
A person signs in with GitHub, installs the ciproxy GitHub App, adds
organizations to a workspace, and follows Setup without its Boat step.
Machines shows the machines of the workspace, and an Admin adds and manages
them there. In Settings, an Admin manages the labels, the organizations, the
workspace name, and the members.
Jobs, Savings, and Plan and usage show a coming-soon state until later work
fills them. The Overflow tab of Settings shows the same state.

The screens follow the [mockups](../../docs/mockups/README.md).
Controls come from `@firna/ui` first, then from `src/components`.
Action buttons and link buttons come from
[src/components/actions.tsx](src/components/actions.tsx). They grow with their
label, so a long name in a label shows in full at each width.
An install link uses the optional `onActivate` callback for a plain click.
Modified clicks keep the browser's navigation to the stable install page.
Colours, fonts, and radii come from the [token package](../tokens/README.md).
The icons and the brand mark are inline SVG elements.

## Pages

The static export holds one HTML file for each route.

| Route                           | Page                                                 |
| ------------------------------- | ---------------------------------------------------- |
| `/`                             | Opens the page that the session asks for             |
| `/sign-in`                      | Sign in with GitHub                                  |
| `/install`                      | The App install, or the wait for an owner's approval |
| `/add-organization`             | The workspace choice and its state after an install  |
| `/personal-account`             | An install on a personal account                     |
| `/app-removed`                  | The App left the last organization of the workspace  |
| `/access-ended`                 | GitHub no longer lists the person                    |
| `/setup`                        | For Admins: the App, a machine, and the first job    |
| `/machines`                     | The machines, one machine, or Add a machine          |
| `/settings`                     | For Admins: labels, organizations, and members       |
| `/jobs`, `/savings`, `/billing` | Coming soon, in the shell                            |

A workspace page names its workspace by its opaque ID in `?workspace=<id>`.
`/?workspace=<id>` opens Setup for an Admin until it is done, else Jobs.
`?reason=github` or `?reason=ended` on `/sign-in` adds a notice.
The shell holds the navigation, the workspace switch, the account menu, and
Add an organization for Admins. A person with no role sees a page that names
the Admins, in place of each workspace page.

Machines and Settings each keep one route, and more query parameters choose
the page. A reload or a link then opens the same page. Each ID is opaque.

| Address                                                        | Page                                              |
| -------------------------------------------------------------- | ------------------------------------------------- |
| `/machines?workspace=<id>`                                     | The machines of the workspace                     |
| `/machines?workspace=<id>&machine=<id>`                        | One machine                                       |
| `/machines?workspace=<id>&view=add`                            | For Admins: Add a machine. It wins over `machine` |
| `/settings?workspace=<id>&tab=<tab>`                           | The tab of Settings                               |
| `/settings?workspace=<id>&tab=organizations&organization=<id>` | The repository access of one organization         |

The tabs are `labels`, `overflow`, `organizations`, and `members`. A missing
or unknown tab opens Labels.

### Machines

- Every role sees the list and each machine. Only an Admin adds, drains, or
  removes a machine, clears its cache, or sets its warm runners and its
  monthly cost. Another role on Add a machine goes to the list.
- The list shows the figures of the machines, a table with the slots,
  health, and cache of each machine, a notice for each offline machine, and
  the Overflow panel with the Boat spend of the month.
- The three pages share one read of `GET /workspaces/{id}/machines`. They
  read it again every 10 seconds, and Add a machine every 5 seconds. A
  machine that the read does not hold shows a not-found state. After Remove,
  the page of the machine goes back to the list.
- Add a machine shows the install command of Setup after its first read of
  the machines. A machine that was not in that read is new, and the page
  shows what its agent reported.
- The machine page shows the saved warm runner count in every state, also
  while the machine drains, is offline, or has no slots. The slots show the
  warm runners that are ready.
- A busy slot names the job that the API matched to it, with its repository,
  its pull request or branch, and how long it has run. The name links to
  `/job?workspace=<id>&job=<id>` with the dashboard ID of the job, a page
  that the Jobs pages own. A job that the API could not match links nowhere,
  and the job ID of a machine report never makes a link.
- The agent version says whether it is the newest release only from a fresh
  report, and only when ciproxy could read the newest release. Cache shows
  the space of the Docker disks and their count of repositories.
- When the API says that the disk holds fewer jobs of `ciproxy` than the
  slots, the page names the limit, the VM disk of one such job, and the disk
  for jobs, and the free slots beyond the limit say that they have no disk
  for a job. Add a machine shows the same limit for a machine that joins.

### Settings

- Settings is for Admins, and another role goes to Jobs. Each tab reads the
  settings of the workspace, so Members shows the count of open requests for
  access on each tab.
- Labels lists each label with its size, where its jobs run now, and the
  jobs of the last 30 days, as the API gives them. A count that the API does
  not know says so.
- Without a Boat account, the jobs of a label run on the machines only, and
  the dialog keeps the fit of the machines and says that Boat is not
  connected. It never says that a Boat size does not fit before the Boat
  catalog says so. When the API knows the fit of the machines but not the
  Boat fit, the list and the dialog give the fit of the machines and say
  that the Boat fit is not known yet.
- While the person types a size or changes a switch in Add label or Edit,
  the dialog asks `POST /workspaces/{id}/labels/preview` again. It shows the
  VM disk of one job, how many jobs each machine runs at a time, and the Boat
  size that runs them, or the offers of the machines and the largest Boat
  size when nothing fits. "Run on the largest Boat size" names that size.
- A label keeps its name. Each label but `ciproxy` has Delete label. When
  GitHub refuses a new name, the dialog shows why.
- Organizations shows the workspace name with Rename, Add an organization,
  and each organization of the workspace. A linked organization has its
  repository access, Take out, and the action that clears a hold: Choose
  Team or Turn on. The last linked organization has no Take out.
- Add an organization is an action of every Admin on every tier. When the
  settings say that a new organization would need Team, the caption under
  the list says that it stays inactive until Team. With one linked
  organization, a notice also offers Team with Compare plans.
- The repository access of an organization is all its private repositories,
  or the selected ones. A public repository cannot be selected.
- When an owner removes the App from one organization on GitHub, its row
  shows "App removed" until ciproxy deletes its data. The row gives the last
  day for a new install that keeps the job history, and Install again. It
  has no repository access, and the rule of the last linked organization
  does not count it.
- On the Organizations tab, the banner of an inactive organization takes the
  place of the soft-limit banner of the shell.
- Members lists the requests for access and the members. The tab gives
  access, declines a request, changes a role, and removes a member. The row
  of the signed-in person has no Remove. Before a person gets the Admin role,
  a dialog shows the trust notice of the API.
- An organization row shows the day it joined and the counts of private
  repositories of its latest complete repository snapshot. An inactive row,
  its banner, and Turn on name the person who added the organization.
- Members shows the name and the login of each person, the day of each
  role, and for each request its day and the organizations that the person
  was a member of then. A day, a name, or a count that the API does not
  know stays out, or the row says that it is not known.

## The API

- The app calls the API under `/api` on its own origin, with the cookies of
  the browser. Serve the export and the API from one origin.
- [@ciproxy/api](../api/README.md) generates every request and response type
  from the Rust code. Names, roles, and choices come from the API only.
- Sign-in is a top-level navigation to `/api/auth/github`. GitHub returns a
  sign-in to `/api/auth/github/callback` and an install to `/api/github/setup`.
  The API checks each return, then opens `/`, or `/sign-in?reason=github` on
  a failure. The app takes no installation ID from a URL.
- The session names the exact claims from the latest successful GitHub
  sign-in, including results kept after a failed authorization. These
  include installs that an owner approved later. When one
  named claim ended, `/` opens its workspace before a remembered one. When
  several ended, `/` opens the return page, which shows each outcome. A
  named claim that still waits adds no route, so the removed-App and
  ended-access pages stay first. The browser handles each ended
  continuation once, and claim order, time, and roles choose nothing.
- The session's install URL opens `/install` without creating an attempt.
  Each install action calls POST `/api/installation-links/start`, then opens
  its returned GitHub URL. The start preserves the session's install purpose.
  Add an organization uses its existing workspace action.
  A plain install action also works after the page observes a newer API build.
- The account menu reads `can_claim_admin` directly from the session.
  The API returns false for a person who already has the Admin role.
- Each API response names its build in the `x-ciproxy-build` header. After a
  new build, the next page change loads a new document, also through the
  back and forward buttons. Until then, the page keeps its input.

## Development

Use the Node version in [.nvmrc](../../.nvmrc).
Run these commands from the repository root:

```sh
npm --prefix ts ci
npm --prefix ts run app:start -- --host localhost
npm --prefix ts run app:typecheck
npm --prefix ts run app:lint
npm --prefix ts --workspace @ciproxy/app run test
npm --prefix ts run app:export
npm --prefix ts --workspace @ciproxy/app run format
```

The development server prints its URL, by default `http://localhost:8081`.
It has no API, so a page that needs a session shows a load failure.
The export goes to `dist/`. The `test` script runs the Node tests in
`_tests_`. The complete gate lints, typechecks, format-checks, and exports
the app. It does not run the Node tests or the browser checks below.

Expo loads the shared Geist faces with `expo-font`, and the Metro
configuration enables their WOFF2 files. The static export puts the faces in
the page head, so the page renders at once. Do not wait for the fonts before
the first render. The client would render nothing where the server rendered
the page, and React would report a hydration error. The export renders each
page without its query, so the first render must not depend on it either.
The page declares the `only light` colour scheme, so a browser that darkens
pages leaves it light.

## Browser checks

The fixture harness serves the export and a typed fixture API on `127.0.0.1`.
It contacts no live API, and runs until Ctrl+C. The browser smoke drives
headless Chrome against it at 1440, 720, and 390 pixels. It covers the pages
and their states, the shell, and the reload after a build change. The
repository has no browser dependency, so supply `playwright-core`. Run the
smoke in a second terminal:

```sh
npm --prefix ts run app:export
node ts/app/scripts/harness-server.ts --port=4340
CIPROXY_PLAYWRIGHT_PATH=/absolute/path/playwright-core/index.mjs \
  node ts/app/scripts/browser-smoke.ts
```

Machines and Settings have a second harness and smoke with the same options.
That smoke covers the Machines pages, the Settings tabs with their dialogs,
and the recovery after a failed read or change. It also needs the export:

```sh
npm --prefix ts run app:export
node ts/app/scripts/m20-harness-server.ts --port=4341
CIPROXY_PLAYWRIGHT_PATH=/absolute/path/playwright-core/index.mjs \
  node ts/app/scripts/m20-browser-smoke.ts
```

- `--case=NAME` runs one case or a comma-separated list of cases. A name
  that matches no case fails the run.
- `CIPROXY_CHROME_PATH` sets the browser, by default `/usr/bin/google-chrome`.
- `CIPROXY_HARNESS_URL` sets the harness, by default `http://127.0.0.1:4340`.
  For Machines and Settings, the default port is 4341.
- `CIPROXY_SMOKE_OUTPUT` sets the directory for the screenshots,
  `results.json`, and a trace of each failed case. By default, it is
  `.context/ciproxy-mvp-m19/browser-smoke` under the current directory. For
  Machines and Settings, it is `.context/ciproxy-mvp-m20/browser-smoke`.
- `POST /__fixture/control` with a JSON `scenario` sets the fixture state of
  a running harness. `scripts/harness-scenarios.ts` and
  `scripts/m20-harness-state.ts` list the scenario names.

## Live check

The live check uses the sign-in of an Admin on a deployed dashboard. It reads
the session, Setup, and the machines, and checks the build header. It creates
one install command and revokes it at once. Then it reloads the Jobs page at
each width. It prints only the name and the result of each check.

The live check of Machines and Settings uses the same credential file. It
reads Machines, the overflow summary, Settings, Members, and the repository
access of one linked organization. It previews an existing label, and
creates and revokes one install command. Then it opens and reloads
Machines, one machine, the Labels, Organizations, and Members tabs, and the
repository access at each width. It changes no machine, label,
organization, repository access, or role. It also prints only the name and
the result of each check.

The credential file holds a live sign-in. Keep it outside the repository,
and never share its values. It is a JSON object with four strings:

- `base_url`: the HTTPS origin of the dashboard, with no path.
- `workspace_id`: a workspace in which the person is an Admin.
- `session_cookie`: the value of the `__Host-ciproxy` cookie of the sign-in.
- `csrf_token`: the `csrf_token` of `GET /api/session` for the sign-in.

```sh
CIPROXY_DASHBOARD_CREDENTIALS_FILE=/absolute/path/outside/repo/dashboard.json \
CIPROXY_PLAYWRIGHT_PATH=/absolute/path/playwright-core/index.mjs \
  node ts/app/scripts/live-smoke.ts
```

Without the credential variable, it prints `SKIP` and exits with status 2.
Run the live check of Machines and Settings the same way:

```sh
CIPROXY_DASHBOARD_CREDENTIALS_FILE=/absolute/path/outside/repo/dashboard.json \
CIPROXY_PLAYWRIGHT_PATH=/absolute/path/playwright-core/index.mjs \
  node ts/app/scripts/m20-live-smoke.ts
```

It prints `SKIP` and exits with status 2 also when the file is missing. A
file that is not valid, or a failed check, exits with status 1. A workspace
with no machine or no linked organization skips the checks that need one.

## Key Code

- [src/app/_layout.tsx](src/app/_layout.tsx) sets the Firna theme, loads the
  fonts, and provides the services. Each other file in `src/app` is a route.
- [src/routing/landing.ts](src/routing/landing.ts) decides where `/` goes.
  [src/routing/claim-continuation.ts](src/routing/claim-continuation.ts)
  reads the exact claims of the latest sign-in.
- [src/screens/workspace-page.tsx](src/screens/workspace-page.tsx) shows a
  workspace page in the [shell](src/components/shell/app-shell.tsx), or the
  page for no role. With `visitKey`, each Machines page and each Settings
  tab checks the session again. With `banner`, a page replaces the
  soft-limit banner of the shell.
- [src/screens/setup/setup-screen.tsx](src/screens/setup/setup-screen.tsx) is
  Setup. `src/screens/add-organization` is the workspace choice.
- The [Machines screen](src/screens/machines/machines-screen.tsx) opens the
  list, one machine, or Add a machine, as
  [src/view/machines-routes.ts](src/view/machines-routes.ts) reads the query.
  The three pages share the [machines store](src/state/machines.ts).
  [src/state/machines-changes.ts](src/state/machines-changes.ts) runs the
  changes of an Admin, one at a time, and
  [src/view/machines-copy.ts](src/view/machines-copy.ts) holds the words.
- The [Settings screen](src/screens/settings/settings-screen.tsx) opens the
  tab that [src/view/settings-tabs.ts](src/view/settings-tabs.ts) reads from
  the query. The [settings store](src/state/settings-store.ts) holds the
  settings of a workspace. A change that succeeds reads them and the session
  again.
- [src/api/client.ts](src/api/client.ts) is the one HTTP client. The
  readers in `src/api/parse-*.ts` narrow each response body to its generated
  type; [src/api/parse-jobs.ts](src/api/parse-jobs.ts) reads one dashboard
  job, as a busy slot names it.
  [src/api/machine-endpoints.ts](src/api/machine-endpoints.ts) and
  [src/api/settings-endpoints.ts](src/api/settings-endpoints.ts) hold the
  routes of Machines and Settings on it.
- [src/components/feature-parts.tsx](src/components/feature-parts.tsx) holds
  the panel, the page header, and the dialog that both features share.
- [src/state/navigation.ts](src/state/navigation.ts) reloads on a new build.
- [app.json](app.json) limits Expo to web and enables static export.

## Related docs

- The [architecture](../../docs/architecture.md#dashboard) of the dashboard.
- [Sign-in and access](../../docs/protocol/security.md#sign-in-and-access),
  [Add an organization](../../docs/protocol/workspaces.md#add-an-organization),
  and [Removing the App](../../docs/protocol/app-removal.md#removing-the-app).
- For Machines: the [install](../../docs/protocol/agent.md#install),
  [health](../../docs/protocol/agent.md#health), and
  [removal](../../docs/protocol/agent.md#drain-and-remove) of a machine, its
  [slots](../../docs/protocol/job-routing.md#slots), and
  [Clear cache](../../docs/protocol/cache.md#cache-scopes).
- For Settings: the [labels](../../docs/protocol/labels.md#custom-labels), the
  [roles](../../docs/protocol/security.md#roles), the
  [repository access](../../docs/protocol/security.md#repository-access), and
  the [organizations](../../docs/protocol/workspaces.md) of a workspace.
- [ciproxy-api](../../crates/ciproxy-api/README.md) registers the GitHub URLs,
  and the [deployment](../../deploy/README.md#environments) serves `/api`.
