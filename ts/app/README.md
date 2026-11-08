# @ciproxy/app

The private ciproxy dashboard, built with Expo Router and React Native Web.
It targets web only and has no open licence.
A person signs in with GitHub, installs the ciproxy GitHub App, adds
organizations to a workspace, and follows Setup without its Boat step.
Jobs shows the live queue, the history of finished jobs, and a page for each
job. Savings compares the cost of the jobs with ciproxy against the same jobs
on Blacksmith and on GitHub-hosted runners. Plan and usage, Plans, and the
Checkout pages show the plan and its use, and take each payment through
Stripe. Machines and Settings show a coming-soon state until later work fills
them.

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

| Route               | Page                                                 |
| ------------------- | ---------------------------------------------------- |
| `/`                 | Opens the page that the session asks for             |
| `/sign-in`          | Sign in with GitHub                                  |
| `/install`          | The App install, or the wait for an owner's approval |
| `/add-organization` | The workspace choice and its state after an install  |
| `/personal-account` | An install on a personal account                     |
| `/app-removed`      | The App left the last organization of the workspace  |
| `/access-ended`     | GitHub no longer lists the person                    |
| `/setup`            | For Admins: the App, a machine, and the first job    |
| `/jobs`             | Jobs: the live queue, or History of finished jobs    |
| `/job`              | One job, opened from History                         |
| `/savings`          | The savings report of one UTC range                  |
| `/billing`          | Plan and usage: the plan, its use, and the add-ons   |
| `/plans`            | The plans, with the offer that covers the use        |
| `/checkout`         | The review of one exact plan choice, then payment    |
| `/checkout-return`  | The return from Stripe Checkout                      |
| `/machines`         | Coming soon, in the shell                            |
| `/settings`         | For Admins: coming soon, in the shell                |

A workspace page names its workspace by its opaque ID in `?workspace=<id>`.
The app takes the ID from the session and never builds one, and the API
checks it on each request.
`/?workspace=<id>` opens Setup for an Admin until it is done, else Jobs.
`?reason=github` or `?reason=ended` on `/sign-in` adds a notice.
The shell holds the navigation, the workspace switch, the account menu, and
Add an organization for Admins. A person with no role sees a page that names
the Admins, in place of each workspace page.

The other choices of a page are in its query too, so a reload or a shared
link keeps them. A value that a page does not know falls back to its default.

- `/jobs`: `tab=history` opens History, with `search=<text>`,
  `place=machine` or `place=boat`, and `days=7`, `days=30`, or `days=90`.
  With no `days`, History shows today. A filter change replaces the address.
- `/job`: `job=<id>` names the job. With no job, the page opens History.
- `/savings`: `range=this_month` or `range=last_month`. With no range, the
  report covers the last 30 days.
- `/checkout`: `tier`, `start_blocks`, and `machines` name one exact choice.
  A missing or invalid value gives no review.
- `/checkout-return`: `session_id=<id>` is the exact Stripe Checkout session.
  The page checks only that session.

Before a tab leaves for Stripe Checkout or the Stripe portal, it saves the
workspace of its page in its own tab storage. `/checkout-return` and `/plans`
without `workspace` use the workspace saved before Checkout, and `/billing`
the one saved before the portal. A query with `workspace` always wins. With
nothing saved, the return page says that it cannot tell the workspace of the
payment. It never guesses one.

Every workspace page shows at most one banner, which the API chooses: a
failed payment, an ended plan, inactive organizations, or a passed limit.
Every role reads Jobs, Savings, and Plan and usage. An action shows only when
the API allows the caller. A person who may not pay reads "Ask an Admin or a
Billing person" in place of each payment action.

## Jobs, Savings, and billing

- Jobs Live reads the queue again every 5 seconds while the tab shows. A
  failed refresh keeps the last queue and says so. A figure that the API
  leaves out says "Unknown", never 0. A waiting job shows its label, not a
  machine, because GitHub gives it to whichever runner is idle first.
- The notices of the queue promise a place only from the facts of each
  waiting label. A full queue counts the jobs whose label fits your
  machines. It promises Boat after the overflow wait only to the labels that
  Boat starts now, and Boat at once to a label that no own machine runs.
  While the Boat costs have no safe count, it promises no Boat start. While
  Boat takes no new job, a notice says that only the jobs whose label fits
  your machines wait there for a free slot.
- A saved Boat retry time holds new Boat starts only while it is after the
  browser time of the read, because the API gives no response time. A
  refused Boat size holds only the labels that use it. A label that no
  permitted place fits names its full request and each place that it
  forbids or that takes no jobs. It never names one short resource, because
  the API does not say which one is. It offers the largest Boat size only
  while the label allows Boat and waits when nothing fits.
- History asks for the finished jobs from the first UTC date of the chosen
  window, and shows every job that the API returns, newest first. The API
  gives no total, so the footer counts the jobs of the list and names the
  history window of the plan in days.
- The job page shows the request of the label as the API kept it, or
  "Requested size unknown", and never the current size of the label. It names
  the VM that ran the job, and why the job ran there when the API gives a
  reason. A job that no runner took is not a start.
- Savings shows the costs that the API gives. The savings figure shows only
  when every cost is known, and a negative saving says "more than". A cost
  that the API knows only in part reads "At least". While the API says that
  its refresh raced another one, the page reads again every 2 seconds, but
  only while it is open. A read that answers after the page closed starts
  no new read.
- Prices, totals, offers, charges, and dates come from the API. The app
  computes no price.
- Plan and usage saves add-ons only at a quantity that the API prices: the
  saved add-ons, or the offer that keeps the plan. Another quantity shows
  "Monthly total not available" and cannot be saved.
- One payment runs at a time. While Stripe confirms a payment or a billing
  change runs, Plan and usage and Plans say so, read again every 5 seconds,
  and start no second payment. A Checkout that stays open on Stripe keeps the
  payment pending until Stripe ends the session.
- While a payment is pending, Plan and usage and Plans offer a person who
  may pay "Continue payment" with the exact Stripe address that this tab
  received from the API, for a Checkout or for the Stripe page of a paid
  change. Another tab or browser has no such address, so its page only says
  that Stripe confirms the payment.
- A payment that waits on its Stripe page says "Your payment is not
  finished", with "Open the Stripe page" and "Check again". It names the
  Stripe page, not a bank, because the API does not say why Stripe waits.
- One payment gate decides whether a payment may start: a payer role in the
  latest session read, a successful latest Plan and usage response, the API
  permission, and no payment or billing change that waits. When the gate is
  closed, Checkout says why: a failed Plan and usage read with Try again,
  before the words of a refused review; a person who cannot pay; a payment
  or a billing change in progress, with "Open plan and usage"; or "Checking
  your plan".
- A payment or a Move to Free whose result the page did not get, such as no
  answer, a server error, or a plain conflict, says only what the next Plan
  and usage read proves. A payment says "Unable to confirm this payment",
  with "Open plan and usage" and Try again, until that read shows the choice
  as the current plan or a payment in progress. The dialog of an unconfirmed
  Move to Free closes with "Close", because the cancellation may have taken
  effect. A refusal that the server names keeps its own words.
- Plan and usage shows each UTC date of the billing period, also its last
  date when the period ends during that day.
- The review shows the charge that the API gives. When the API cannot price a
  change, the review says so, and no payment starts.
- Every payment request goes through one module. It sends the reviewed or
  quoted choice, never the quantities that the person still edits.
- The return page checks the session at once, and every 3 seconds while
  Stripe confirms. Only a confirmation from the API changes the plan on the
  page. When Stripe needs the person again, the page links to the Stripe
  address that the API gives. After a confirmed payment, only an Admin gets
  Add machines, and a Billing person gets only Open plan and usage.
- Jobs, the job page, Savings, Plan and usage, Plans, the Checkout review,
  and the return page take the banner from their own reads. An older read
  never replaces the banner of a newer one. A failed read keeps the last
  banner, and the page shows its own failure. Other pages, the Organizations
  scope, and a Checkout or return page that reads nothing use the banner
  endpoint, unless a read just ended. There, a failure shows "Unable to
  check the plan status", and a successful page read ends that failure.

## The API

- The app calls the API under `/api` on its own origin, with the cookies of
  the browser. Serve the export and the API from one origin.
- [@ciproxy/api](../api/README.md) generates every request and response type
  from the Rust code. Names, roles, and choices come from the API only.
- Every read and change goes through the one HTTP client. Each read passes
  through the session store, so an ended sign-in opens sign-in, and a refused
  role reads the session again.
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
  back and forward buttons. Until then, the page keeps its input. A filter or
  range change after a new build loads a new document with the new address.
- The billing runtime has one Checkout success URL, one cancel URL, and one
  portal return URL for all workspaces, with no workspace placeholder. Set
  the success URL to `/checkout-return?session_id={CHECKOUT_SESSION_ID}` on
  the dashboard origin, so that Stripe names the exact session. The return
  then takes the workspace that its tab saved.

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

The fixture harness serves the export and a typed fixture API on `127.0.0.1`,
with no live API. The browser smokes drive headless Chrome against it at
1440, 720, and 390 pixels: the shell smoke checks sign-in, Setup, and the
shell, and the page smoke checks Jobs, Savings, and the billing pages.
[Dashboard scripts](scripts/README.md#browser-checks) gives the commands, the
flags, the scenarios, and the evidence folders.

## Live check

The live checks read a deployed dashboard with the sign-in of a person, from
a private credential file outside the repository. Without
`CIPROXY_DASHBOARD_CREDENTIALS_FILE`, each check prints `SKIP` and exits with
status 2. [Dashboard scripts](scripts/README.md#live-checks) gives the
credential file, the steps of the page check, and its private result file.

## Key Code

- [src/app/_layout.tsx](src/app/_layout.tsx) sets the Firna theme, loads the
  fonts, and provides the services. Each other file in `src/app` is a route.
- [src/routing/landing.ts](src/routing/landing.ts) decides where `/` goes.
  [src/routing/claim-continuation.ts](src/routing/claim-continuation.ts)
  reads the exact claims of the latest sign-in.
- [src/screens/workspace-page.tsx](src/screens/workspace-page.tsx) shows a
  workspace page in the [shell](src/components/shell/app-shell.tsx), or the
  page for no role.
- [src/screens/setup/setup-screen.tsx](src/screens/setup/setup-screen.tsx) is
  Setup. `src/screens/add-organization` is the workspace choice.
- [src/screens/jobs](src/screens/jobs) holds Jobs, History, and the job page.
  [src/screens/savings](src/screens/savings) holds Savings, and
  [src/screens/billing](src/screens/billing) Plan and usage, Plans, and the
  Checkout pages.
- [src/view/jobs-links.ts](src/view/jobs-links.ts) builds the Jobs addresses
  and the History query.
  [src/view/jobs-places.ts](src/view/jobs-places.ts) gives where the waiting
  jobs of each label can start, and the one hold of Boat at the time of the
  read, which every notice of the queue reads.
  [src/view/jobs-label-notices.ts](src/view/jobs-label-notices.ts) words the
  notices of each waiting label from those facts.
- [src/state/savings.ts](src/state/savings.ts) reads the Savings report. Its
  `open()` holds the page lifetime, so the report reads again only while the
  page is open.
- [src/state/billing.ts](src/state/billing.ts) creates the billing stores of
  a workspace. [src/state/billing-banner.ts](src/state/billing-banner.ts)
  keeps the one banner, and its `readPage` takes the banner of each page read.
  [src/api/billing-payment.ts](src/api/billing-payment.ts) sends every payment
  request.
- The [workspace banner](src/components/billing/workspace-banner.tsx) in
  `src/components/billing` shows the one banner of each workspace page.
- [src/components/panel.tsx](src/components/panel.tsx) and its neighbours
  `figures.tsx`, `detail-list.tsx`, `link-tabs.tsx`, and `status-label.tsx`
  are the shared page parts.
  [src/view/exact-format.ts](src/view/exact-format.ts) formats counts, cents,
  and UTC dates without floating-point loss.
- [src/api/client.ts](src/api/client.ts) is the one HTTP client.
  [src/api/route-path.ts](src/api/route-path.ts) builds the path of a
  generated route.
- [src/state/navigation.ts](src/state/navigation.ts) reloads on a new build.
- [scripts](scripts/README.md) holds the fixture harness, the browser smokes,
  and the live checks.
- [app.json](app.json) limits Expo to web and enables static export.

## Related docs

- The [architecture](../../docs/architecture.md#dashboard) of the dashboard.
- [Sign-in and access](../../docs/protocol/security.md#sign-in-and-access),
  [Add an organization](../../docs/protocol/workspaces.md#add-an-organization),
  and [Removing the App](../../docs/protocol/app-removal.md#removing-the-app).
- [Job routing](../../docs/protocol/job-routing.md#what-the-dashboard-shows),
  the [savings report](../../docs/protocol/metering.md#savings-report), and
  [Checkout](../../docs/protocol/billing.md#checkout).
- [ciproxy-api](../../crates/ciproxy-api/README.md) registers the GitHub URLs,
  and the [deployment](../../deploy/README.md#environments) serves `/api`.
