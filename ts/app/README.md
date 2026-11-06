# @ciproxy/app

The private ciproxy dashboard, built with Expo Router and React Native Web.
It targets web only and has no open licence.
A person signs in with GitHub, installs the ciproxy GitHub App, adds
organizations to a workspace, and follows Setup without its Boat step.
Jobs, Machines, Savings, Plan and usage, and Settings show a coming-soon
state until later work fills them.

The screens follow the [mockups](../../docs/mockups/README.md).
Controls come from `@firna/ui` first, then from `src/components`.
Colours, fonts, and radii come from the [token package](../tokens/README.md).
The icons and the brand mark are inline SVG elements.

## Pages

The static export holds one HTML file for each route.

| Route                                        | Page                                                 |
| -------------------------------------------- | ---------------------------------------------------- |
| `/`                                          | Opens the page that the session asks for             |
| `/sign-in`                                   | Sign in with GitHub                                  |
| `/install`                                   | The App install, or the wait for an owner's approval |
| `/add-organization`                          | The workspace choice and its state after an install  |
| `/personal-account`                          | An install on a personal account                     |
| `/app-removed`                               | The App left the last organization of the workspace  |
| `/access-ended`                              | GitHub no longer lists the person                    |
| `/setup`                                     | For Admins: the App, a machine, and the first job    |
| `/jobs`, `/machines`, `/savings`, `/billing` | Coming soon, in the shell                            |
| `/settings`                                  | For Admins: coming soon, in the shell                |

A workspace page names its workspace by its opaque ID in `?workspace=<id>`.
`/?workspace=<id>` opens Setup for an Admin until it is done, else Jobs.
`?reason=github` or `?reason=ended` on `/sign-in` adds a notice.
The shell holds the navigation, the workspace switch, the account menu, and
Add an organization for Admins. A person with no role sees a page that names
the Admins, in place of each workspace page.

## The API

- The app calls the API under `/api` on its own origin, with the cookies of
  the browser. Serve the export and the API from one origin.
- [@ciproxy/api](../api/README.md) generates every request and response type
  from the Rust code. Names, roles, and choices come from the API only.
- Sign-in is a top-level navigation to `/api/auth/github`. GitHub returns a
  sign-in to `/api/auth/github/callback` and an install to `/api/github/setup`.
  The API checks each return, then opens `/`, or `/sign-in?reason=github` on
  a failure. The app takes no installation ID from a URL.
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

- `--case=NAME` runs one case, such as `sign-in` or `build-history`. A name
  that matches no case fails the run.
- `CIPROXY_CHROME_PATH` sets the browser, by default `/usr/bin/google-chrome`.
- `CIPROXY_HARNESS_URL` sets the harness, by default `http://127.0.0.1:4340`.
- `CIPROXY_SMOKE_OUTPUT` sets the directory for the screenshots,
  `results.json`, and a trace of each failed case. By default, it is
  `.context/ciproxy-mvp-m19/browser-smoke` under the current directory.

## Live check

The live check uses the sign-in of an Admin on a deployed dashboard. It reads
the session, Setup, and the machines, and checks the build header. It creates
one install command and revokes it at once. Then it reloads the Jobs page at
each width. It prints only the name and the result of each check.

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

## Key Code

- [src/app/_layout.tsx](src/app/_layout.tsx) sets the Firna theme, loads the
  fonts, and provides the services. Each other file in `src/app` is a route.
- [src/routing/landing.ts](src/routing/landing.ts) decides where `/` goes.
- [src/screens/workspace-page.tsx](src/screens/workspace-page.tsx) shows a
  workspace page in the [shell](src/components/shell/app-shell.tsx), or the
  page for no role.
- [src/screens/setup/setup-screen.tsx](src/screens/setup/setup-screen.tsx) is
  Setup. `src/screens/add-organization` is the workspace choice.
- [src/api/client.ts](src/api/client.ts) is the one HTTP client.
- [src/state/navigation.ts](src/state/navigation.ts) reloads on a new build.
- [app.json](app.json) limits Expo to web and enables static export.

## Related docs

- The [architecture](../../docs/architecture.md#dashboard) of the dashboard.
- [Sign-in and access](../../docs/protocol/security.md#sign-in-and-access),
  [Add an organization](../../docs/protocol/workspaces.md#add-an-organization),
  and [Removing the App](../../docs/protocol/app-removal.md#removing-the-app).
- [ciproxy-api](../../crates/ciproxy-api/README.md) registers the GitHub URLs,
  and the [deployment](../../deploy/README.md#environments) serves `/api`.
