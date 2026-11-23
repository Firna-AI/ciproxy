# @ciproxy/app

The private ciproxy dashboard, built with Expo Router and React Native Web.
It targets web only and has no open licence.
A person signs in with GitHub, installs the ciproxy GitHub App, adds
organizations to a workspace, and follows Setup. Its Boat step connects the
Boat account of the customer, which takes jobs when the machines of the
workspace are busy.
Machines shows the machines of the workspace, and an Admin adds and manages
them there. In Settings, an Admin manages the labels, the organizations, the
workspace name, and the members. On Settings under Overflow, an Admin
connects Boat, changes its key, or disconnects it. The same page sets the
overflow switch, the wait, and the monthly spend limit, and shows the spend
of the month.
Jobs shows the live queue, the history of finished jobs, and a page for each
job with its CPU and memory. Savings compares the cost of the jobs with
ciproxy against the same jobs on Blacksmith and on GitHub-hosted runners.
Plan and usage, Plans, and the Checkout pages show the plan and its use, and
take each payment through Stripe.

The screens follow the [mockups](../../docs/mockups/README.md).
Controls come from `@firna/ui` first, then from `src/components`.
Action buttons and link buttons come from
[src/components/actions.tsx](src/components/actions.tsx). They grow with their
label, so a long name in a label shows in full at each width.
An install link uses the optional `onActivate` callback for a plain click.
Modified clicks keep the browser's navigation to the stable install page.
A link inside a sentence, such as Terms, comes from
[src/components/inline-link.tsx](src/components/inline-link.tsx). It stays
underlined. In a dialog or a review, it opens a new tab.
Colours, fonts, and radii come from the [token package](../tokens/README.md).
The icons and the brand mark are inline SVG elements. The favicon is the
brand mark in the ink, as a data URL from
[src/components/brand.tsx](src/components/brand.tsx).

## Pages

The static export holds one HTML file for each route.

| Route                | Page                                                 |
| -------------------- | ---------------------------------------------------- |
| `/`                  | Opens the page that the session asks for             |
| `/sign-in`           | Sign in with GitHub                                  |
| `/continue`          | The terms step before a GitHub sign-in               |
| `/install`           | The App install, or the wait for an owner's approval |
| `/add-organization`  | The workspace choice and its state after an install  |
| `/personal-account`  | An install on a personal account                     |
| `/app-removed`       | The App left the last organization of the workspace  |
| `/access-ended`      | GitHub no longer lists the person                    |
| `/setup`             | For Admins: the App, a machine, Boat, the first job  |
| `/machines`          | The machines, one machine, or Add a machine          |
| `/settings`          | For Admins: labels, organizations, and members       |
| `/settings/overflow` | For Admins: Boat, the overflow wait, the spend limit |
| `/jobs`              | Jobs: the live queue, or History of finished jobs    |
| `/job`               | One job, opened from History                         |
| `/savings`           | The savings report of one UTC range                  |
| `/billing`           | Plan and usage: the plan, its use, and the add-ons   |
| `/plans`             | The plans, with the offer that covers the use        |
| `/checkout`          | The review of one exact plan choice, then payment    |
| `/checkout-return`   | The return from Stripe Checkout                      |

A workspace page names its workspace by its opaque ID in `?workspace=<id>`.
The app takes the ID from the session and never builds one, and the API
checks it on each request.
`/?workspace=<id>` opens Setup for an Admin until it is done, else Jobs.
Setup is done after the App, a machine, and the first job, with or without
Boat. After Skip for now, the Boat step links to Settings under Overflow.
The API keeps no record of a skip, so it lasts only for the tab, the
workspace, and the person who skipped.
While the machine of Setup gets its first job image, its step says so, and
Setup goes on. A first job that waits for a runner then says that it waits
for that machine, while no other machine takes jobs and Boat is not
connected. Setup reads again while a machine gets its image, also after the
first job ended.
`?reason=github` or `?reason=ended` on `/sign-in` adds a notice.
The shell holds the navigation, the workspace switch, the account menu, and
Add an organization for Admins. A person with no role sees a page that names
the Admins, in place of each workspace page.

Machines and Settings each keep one route, and more query parameters choose
the page. A reload or a link then opens the same page. Each ID is opaque.
Settings under Overflow has a route of its own.

| Address                                                        | Page                                              |
| -------------------------------------------------------------- | ------------------------------------------------- |
| `/machines?workspace=<id>`                                     | The machines of the workspace                     |
| `/machines?workspace=<id>&machine=<id>`                        | One machine                                       |
| `/machines?workspace=<id>&view=add`                            | For Admins: Add a machine. It wins over `machine` |
| `/settings?workspace=<id>&tab=<tab>`                           | The tab of Settings                               |
| `/settings?workspace=<id>&tab=organizations&organization=<id>` | The repository access of one organization         |
| `/settings/overflow?workspace=<id>`                            | For Admins: Settings under Overflow               |

The tabs are `labels`, `overflow`, `organizations`, and `members`. A missing
or unknown tab opens Labels. The Overflow tab opens Settings under Overflow.
The address with `tab=overflow` opens that page too, in place of its own
entry in the browser history. The tabs of Settings under Overflow open the
other tabs on `/settings`.

The other choices of a page are in its query too, so a reload or a shared
link keeps them. A value that a page does not know falls back to its default.

- `/jobs`: `tab=history` opens History, with `search=<text>`,
  `place=machine` or `place=boat`, and `days=7`, `days=30`, or `days=90`.
  With no `days`, History shows today. A tab change adds an entry to the
  browser history, so Back opens the last tab. A filter change replaces the
  address.
- `/job`: `job=<id>` names the job. With no job, the page opens History.
- `/savings`: `range=this_month` or `range=last_month`. With no range, the
  report covers the last 30 days.
- `/checkout`: `tier`, `start_blocks`, and `machines` name one exact choice.
  A missing or invalid value gives no review.
- `/checkout-return`: `session_id=<id>` is the exact Stripe Checkout session.
  The page checks only that session.

Every workspace page shows at most one banner, which the API chooses: a
failed payment, an ended plan, inactive organizations, or a passed limit.
Every role reads Jobs, Savings, and Plan and usage. An action shows only when
the API allows the caller. A person who may not pay reads "Ask an Admin or a
Billing person" in place of each payment action.

### Machines

[Machines pages](src/screens/machines/README.md) gives the rules of the list,
one machine, and Add a machine: the roles, the reads, the slots, and the disk
limit.

### Settings

[Settings pages](src/screens/settings/README.md) gives the rules of the
Labels, Organizations, and Members tabs, the label dialog, and Settings under
Overflow with its Boat form.

- On the Organizations tab, the shell shows the banner of the Organizations
  scope, in which inactive organizations come before a passed limit. For one
  inactive organization, that banner gives its reasons in the words of its
  row. For more, it points to the rows. It offers only what the API allows,
  such as Choose Team, or Turn on with the dialog of the row.

## Jobs, Savings, and billing

- [Jobs pages](guides/jobs/README.md) gives the rules of Jobs Live, its
  notices, History, and the job page.
- Savings shows the costs that the API gives. The savings figure shows only
  when every cost is known, and a negative saving says "more than". A cost
  that the API knows only in part reads "At least". While the API says that
  its refresh raced another one, the page reads again every 2 seconds, but
  only while it is open. A read that answers after the page closed starts
  no new read.
- [Billing pages](src/screens/billing/README.md) gives the rules of Plan and
  usage, Plans, the Checkout review, each payment, and the Stripe returns.
- Jobs, the job page, Savings, Plan and usage, Plans, the Checkout review,
  and the return page take the banner from their own reads. An older read
  never replaces the banner of a newer one. A failed read keeps the last
  banner, and the page shows its own failure. Other pages, the Organizations
  scope, and a Checkout or return page that reads nothing use the banner
  endpoint. There, a failure shows "Unable to check the plan status", and a
  successful page read ends that failure. A page reuses a read of the
  endpoint that just ended, but the Organizations scope reads at each entry.
  The scope reads again after each Settings read that changes the
  organizations or their holds, such as after Turn on.

## The API

- The app calls the API under `/api` on its own origin, with the cookies of
  the browser. Serve the export and the API from one origin.
- [@ciproxy/api](../api/README.md) generates every request and response type
  from the Rust code. Names, roles, and choices come from the API only.
- Every read and change goes through the one HTTP client. Each read passes
  through the session store, so an ended sign-in opens sign-in, and a refused
  role reads the session again.
- [Dashboard API use](src/api/README.md) gives the rules of sign-in, the
  claims of the session, the install, the account menu, and a new API build.
- The [Settings guide](src/screens/settings/README.md#settings-under-overflow)
  gives the rules of the Boat settings and of the Boat form.

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

[src/components/fonts.tsx](src/components/fonts.tsx) writes the
`@font-face` rules and the preloads of the shared Geist faces into the page
head. The Metro configuration enables their WOFF2 files. Each rule declares
the weight range of the variable file, so Safari adds no synthetic bold.
`expo-font` writes no weight range, so the app does not use it. The static
export puts the rules in the page head, so the page renders at once.
Do not wait for the fonts before the first render. The client would render
nothing where the server rendered the page, and React would report a
hydration error. The export renders each
page without its query, so the first render must not depend on it either.
The page declares the `only light` colour scheme, so a browser that darkens
pages leaves it light.

## Browser checks

Three browser smokes drive headless Chrome against a fixture harness, with
no live API: the shell smoke, the page smoke, and the Machines and Settings
smoke. [Dashboard scripts](scripts/README.md#browser-checks) gives what each
smoke checks, the commands, the flags, the scenarios, and the evidence
folders.

## Live checks

Four live checks read a deployed dashboard with the sign-in of a person, from
a private credential file outside the repository: the shell check, the
Machines and Settings check, the page check, and the Boat check.
[Dashboard scripts](scripts/README.md#live-checks) gives the credential file,
the command and the exit status of each check, and the private result file
of the page check.

### Boat settings

[Dashboard scripts](scripts/README.md#boat-live-check) gives the Boat live
check, which reads the Boat settings of a deployed dashboard.

## Key Code

- [src/app/_layout.tsx](src/app/_layout.tsx) sets the Firna theme, loads the
  fonts, and provides the services. Each other file in `src/app` is a route.
- [src/routing/landing.ts](src/routing/landing.ts) decides where `/` goes.
  [src/routing/claim-continuation.ts](src/routing/claim-continuation.ts)
  reads the exact claims of the latest sign-in.
- [src/screens/workspace-page.tsx](src/screens/workspace-page.tsx) shows a
  workspace page in the [shell](src/components/shell/app-shell.tsx), or the
  page for no role. With `visitKey`, each Machines page and each Settings
  tab checks the session again. With `bannerScope`, the Organizations tab
  asks the shell for the banner of its scope.
- [src/screens/setup/setup-screen.tsx](src/screens/setup/setup-screen.tsx) is
  Setup, with its Boat step in `boat-step.tsx`.
  `src/screens/add-organization` is the workspace choice.
- The guides of [Machines](src/screens/machines/README.md#key-code),
  [Settings](src/screens/settings/README.md#key-code),
  [Jobs](guides/jobs/README.md#key-code),
  [billing](src/screens/billing/README.md#key-code), and
  [API use](src/api/README.md#key-code) give the entry points of their code.
- [src/screens/savings](src/screens/savings) holds Savings, and
  [src/screens/billing](src/screens/billing) Plan and usage, Plans, and the
  Checkout pages.
- [src/state/savings.ts](src/state/savings.ts) reads the Savings report. Its
  `open()` holds the page lifetime, so the report reads again only while the
  page is open.
- [src/state/page-store.ts](src/state/page-store.ts) keeps the page of one
  read, such as Plan and usage, Plans, or the overflow summary of Machines.
  A newer read wins over an older one, and a failed refresh keeps the last
  good page with the failure. A watched page store reads again on its
  interval while its page waits, or while no good page exists.
- [src/state/billing.ts](src/state/billing.ts) creates the billing stores of
  a workspace. [src/state/billing-banner.ts](src/state/billing-banner.ts)
  keeps the one banner, and its `readPage` takes the banner of each page read.
  [src/api/billing-payment.ts](src/api/billing-payment.ts) sends every payment
  request.
- The [workspace banner](src/components/billing/workspace-banner.tsx) in
  `src/components/billing` shows the one banner of each workspace page. Its
  `turn-on-action.tsx` opens the Turn on dialog of the Organizations tab.
  [src/state/organizations-banner.ts](src/state/organizations-banner.ts)
  reads the banner of that scope at each entry and after each change of its
  organizations.
- [src/components/panel.tsx](src/components/panel.tsx) and its neighbours
  `figures.tsx`, `detail-list.tsx`, `link-tabs.tsx`, and `status-label.tsx`
  are the shared page parts.
  [src/view/exact-format.ts](src/view/exact-format.ts) formats counts, cents,
  and UTC dates without floating-point loss.
- [src/components/feature-parts.tsx](src/components/feature-parts.tsx) holds
  the panel, the page header, and the dialog that both features share.
- [scripts](scripts/README.md) holds the fixture harnesses, the browser
  smokes, and the live checks.
- [app.json](app.json) limits Expo to web and enables static export.

## Related docs

- The [architecture](../../docs/architecture.md#dashboard) of the dashboard.
- [Sign-in and access](../../docs/protocol/security.md#sign-in-and-access),
  [Add an organization](../../docs/protocol/workspaces.md#add-an-organization),
  and [Removing the App](../../docs/protocol/app-removal.md#removing-the-app).
- [Overflow](../../docs/protocol/overflow.md), with the Boat account and the
  spend limit, and
  [the overflow wait](../../docs/protocol/job-routing.md#the-overflow-wait).
- [Job routing](../../docs/protocol/job-routing.md#what-the-dashboard-shows),
  the [savings report](../../docs/protocol/metering.md#savings-report), and
  [Checkout](../../docs/protocol/billing.md#checkout).
- [ciproxy-api](../../crates/ciproxy-api/README.md) registers the GitHub URLs,
  and the [deployment](../../deploy/README.md#environments) serves `/api`.
- Each page guide above lists the protocol docs of its own pages.
