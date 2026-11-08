# Mockups

The mockups of ciproxy: the hosted dashboard, and the public site with the
marketing pages and the docs. They are part of the spec: each screen of the MVP
renders in a mobile and a desktop variant, with the empty, loading, and error
states that it needs, and the user flows step through those screens.
[Mokly](https://github.com/mokly-ai/mokly) builds the catalogue from React entry
modules.

## Commands

Use the Node version in [.nvmrc](../../.nvmrc).
The `.nvmrc` file must name a Node version that Mokly accepts.
Mokly rejects Node older than 22.15, Node 23, and Node 24.14 to 24.18.
Run these commands from the repository root:

```sh
npm --prefix ts ci                   # install the workspace
npm --prefix ts run mockups:build    # strict build into mokly-generated
npm --prefix ts run mockups:check    # build, types, lint, format, policy
npm --prefix ts run mockups:serve    # browse, with changes against main
npm --prefix ts run mockups:format   # format the mockup sources with Prettier
```

`mockups:check` fails on any of these:

- A Mokly build error or warning: missing metadata, a link or a flow step to a
  path with no entry, a flow that a step's screen does not list in
  `useCasePaths`, or a folder order that names no child.
- A TypeScript error under the strict configuration in `tsconfig.json`.
- An ESLint finding of the `typescript-eslint` recommended rules, or a file
  that Prettier would change.
- A policy violation in `scripts/check-policy.ts`: a page with more than five
  screens, a page with sub-pages that does not render exactly one canonical
  screen, a source file over 300 lines, or generated output that Git tracks or
  does not ignore.

## Catalogue rules

- One screen is one component under `src/screens/<area>/`. It takes the
  viewport from the Mokly render context and renders both variants. A state is
  a prop of the screen, not a copy of its markup.
- A page shows at most five screens: an entry with its variants counts in full.
  When an area needs more, it gets an `index.mockup.tsx` with its one canonical
  screen, and its states move into child folders.
- A user flow under `src/entries/user-flows/` steps only through screens that
  exist. Each step's screen lists the flow in `useCasePaths`; a variant lists
  it on the variant.
- Every link and flow step uses a path from `src/components/paths.ts`. A screen
  links with `MockLink`, through `LinkAction` and `TextLink` for buttons and
  text links.
- Notes for reviewers go in the `description` and `rationale` of an entry,
  never inside the rendered screen.
- Each entry names the protocol docs of its screens in `relatedDocs`, through
  `related()` of `src/components/related-docs.ts`, which adds the plan. An
  entry that shows a design call or an answer of the MVP plan uses
  `relatedMvp()`, which adds that plan too.
- Fixture data lives in `src/fixtures/`. It tells one story: mokly-ai signs up
  on 1 September 2026, passes the Free starts on 4 September, moves to Team
  that day, and runs three Hetzner AX42 machines with Boat overflow from
  mid-September. Its workspace takes the name mokly-ai, and adds a second
  organization, mokly-labs, on 2 November. The November screens show the busy
  Team workspace, and the December screens the payment problems and a move to
  Free that keeps both organizations. Each tier counts only the starts of its
  own period, which begins on the day the tier begins. Cal Moore installs the
  GitHub App, so he is the first Admin. `members.ts` names the person of each
  role, the owner with the Billing role, and a second Admin.
- Some screens are what-ifs of November, in `access.ts` and `machines.ts`: a
  fourth machine, the mini PC office-mini-1, whose disk holds fewer jobs than
  its slots; Tom Becker suspends or removes the GitHub App; and Jonas Weber
  leaves the organizations on GitHub. `hardware.ts` holds the slot rule and
  the disk rule, in which each running job holds its VM disk: the current
  image with the free disk of its label, as `vm-disk.ts` gives it. In
  `job-details.ts`, ciproxy has no saved request for two jobs of the history,
  so their pages show an unknown request and the VM that ran each job.
- The return of Add an organization and the inactive organizations are
  what-ifs in `org-links.ts`. In the main story, Cal Moore is an Admin of one
  workspace, so the return links mokly-labs at once. On 2 November, he is also
  an Admin of driftwood, a workspace on Free, so the return lists two. On 8
  December, on Free, a third organization, mokly-studio, joins mokly-ai
  inactive. In a second what-if, Ana Souza adds it, and her Admin role ends
  after the return saves its workspace and before ciproxy finishes the link,
  so it joins with a second hold, which waits for an Admin. Every billing link
  of 8 December opens the Plan and usage and the Plans of that day, as
  `src/screens/billing/billing-scenes.tsx` joins them.
- The overflow flow and the Savings report share one day: `releaseDay` of
  `live-jobs.ts`, the release of 18 November. The report covers the 30 UTC
  days that end on it, and its range and day labels come from it.
- The known-data scenes of the story show each page with every fact, but a
  banner names only the facts that the API gives. Beside them, a
  `current-contract` folder shows the same moments with only the facts that
  the API gives. In Jobs, it names no person or date of a suspension, no
  reason of a Boat refusal, no total of the day, no tier name, no routing
  reason, no fit of a Boat size, and no resource that is short for a label
  that nothing fits. In Savings, it names no tier and no machine count. In
  billing, it names no unit prices of the add-ons, no receipt address, and
  no workspace in a Stripe return. Its `pending` folder shows
  Continue payment for a payment whose Stripe page the tab saved. Its
  `checkout` folder shows a paid review, an unavailable price, and a refused
  review that asks the person to review the change again. Its `payment-gate`
  folder shows why a payment cannot start, and `unknown-results` shows a
  payment or a Move to Free whose result the page did not get. A payment
  that waits on its Stripe page names that page, not a bank.
- The Jobs folders `wait-facts`, `queue-facts`, `place-facts`, `fit-facts`,
  and `job-states` and the Savings folder `report-states` show the states
  that come from facts of the API, such as an unknown fit, a job that no
  runner took, a negative saving, and a missing comparison price.
  `place-facts` shows where the waiting jobs of each label can start, also
  of a label that only Boat runs and of a label whose Boat size Boat
  refused. `fit-facts` shows a label that no permitted place fits, with its
  full request, and a label that does not run on Boat.
- A screen takes the role of the signed-in person. The shell shows the person
  of the role, and Settings only to an Admin. In place of an action that the
  role does not allow, the screen shows the words of
  `src/components/role-actions.tsx`, such as "Ask an Admin".
- The public site has two sections: Marketing, with the landing page, the
  pricing page, and the frames of the terms and the privacy policy, and Docs,
  in the structure of Starlight: the sidebar of
  sections, the top bar with search, the article, and "On this page". Both
  share the site top bar, on the domain `ciproxy.dev`. Every button that opens
  sign-in says "Start free". The landing page shows no customer logos, quotes,
  or press. Its cost claim uses no sample of an organization: `cost-claim.ts`
  computes every number from the list prices in `price-lists.ts`, each with
  its source and the day it was checked.
- Each tier feature has one record in `tiers.ts`, with its first tier and
  whether it ships at launch. Every page that lists a feature reads the record,
  and a feature that ships later says "Coming later", through
  `src/components/tier-feature-list.tsx`.
- One value has one format: `src/fixtures/format.ts` holds the money, count,
  and date formats, and `unbroken()` keeps a short value such as a date on one
  line.
- The look is Graphite: light and monochrome, Geist and Geist Mono, hairline
  borders, small radii, and black primary buttons. Colour marks status only,
  as a dot beside its word. Its tokens live in
  [`@ciproxy/tokens`](../../ts/tokens/README.md), and a stylesheet reads them
  as custom properties, never as colour literals.
- Check `@firna/ui` and `src/components/` before you build a control. The Firna
  DOM build renders without React Native; the renderer adds its static CSS and
  the Graphite theme.
- A banner of a passed limit or a failed payment comes from
  `src/components/usage-banner.tsx`, so one state shows one banner on every
  page. On Free, a second organization passes a soft limit too. At a failed
  payment, the payment banner takes the place of the soft-limit banner.
- In the Organizations tab, an inactive organization gets the banner of
  `src/components/inactive-banner.tsx`, which says why and offers Team or Turn
  on. On Free, it takes the place of the soft-limit banner of the
  organizations, and at a failed payment, the payment banner takes its place.
- The legal pages show grey lines in place of the text that the user writes
  or approves before the launch. Only the data retention of the privacy policy
  has text, from `legal.ts`.
- Never edit or commit `docs/mockups/mokly-generated/`. Change the sources and
  build again.

## Words

- "Workspace" is the customer of ciproxy: one or more GitHub organizations
  with one tier, members, machines, labels, settings, and Boat account. Never
  write "team" for it, because Team is a tier.
- "Machine" is the customer's own hardware, on every screen and in the
  pricing. Never write "node" or "server", and never call a VM a machine.
- "Size" is what a label gives a job: its vCPU, memory, and free disk. Write
  "VM size" where the word needs a qualifier. Never write "shape" or "machine
  size". Code names use the same words, such as `sizeText` and `LabelDialog`.
- "Free disk" is the disk of a label: the space that each job gets, as GitHub
  states the disk of its runners. "VM disk" is the whole disk of a job VM: the
  image with the free disk. A Boat size states its whole disk. Write "free
  disk" or "VM disk" where a reader could mix them up, and use the same words
  in code, such as `freeDiskGb` and `vmDiskBytes`.

## Visual smoke test

1. Run `npm --prefix ts run mockups:check`.
2. Open each changed screen at
   `docs/mockups/mokly-generated/static/mokly-generated/<path>/index.mobile.html`
   and `index.desktop.html`. Check the desktop document at 1440 px and at
   720 px: the catalogue shows the desktop frame at about 720 px beside the
   phone, where the sidebar becomes an icon rail. Check the mobile document at
   390 px.
3. Serve the build, for example with
   `python3 -m http.server --directory docs/mockups/mokly-generated`, and open
   `/view/user-flows/<flow>/` for each changed flow. Each step frames its
   screen and links back to it.
4. Look for clipped or overlapping text, a broken layout at either width, and
   copy that does not follow the `product-ui` skill. Keep screenshots under
   `.context/`.

## Development

### Key Code

- `../../mokly.config.json`: the catalogue root, the renderer, and the package
  root `ts` for module resolution. Mokly 0.18 reads only a JSON configuration.
- `src/render.tsx`: the renderer. It adds the base stylesheet, the static
  Firna CSS with the tokens, and the Firna theme provider around every screen.
- [`@ciproxy/tokens`](../../ts/tokens/README.md): the Graphite tokens: the
  Firna theme with the Geist faces, and the status, chart, slot, and sidebar
  colours.
- `src/theme/firna.tsx`: takes the `--c-*`, `--r-*`, and `--font-*` custom
  properties from `tokenDeclarations()` of `@ciproxy/tokens/css`, adds the
  two Firna focus-ring properties, and puts them beside the static Firna CSS.
- `src/components/site/site-header.tsx`: the top bar of the public site,
  shared by the marketing pages and the docs, with the search box on the docs.
- `src/screens/marketing/marketing-shell.tsx`: the frame of a marketing page:
  the sections between two hairline rails, and the footer. `section.tsx` holds
  the section head and the cells divided by hairlines. `legal-page.tsx` is the
  frame of the terms and the privacy policy, with their sections from
  `src/fixtures/legal.ts`.
- `src/screens/docs/docs-shell.tsx`: the frame of a docs page, in the structure
  of Starlight, with the sidebar from `docs-nav.ts` and the article parts in
  `docs-parts.tsx`.
- `src/components/shell/app-shell.tsx`: the signed-in frame: a white sidebar
  and a top bar with the workspace switch and the page as breadcrumbs on
  desktop, and a top bar with tabs on mobile, with the banner slot for
  soft-limit and payment warnings. It shows the signed-in person, and
  Settings only to an Admin. The switch opens the workspaces of
  `workspace-menu.tsx`. The avatar opens the account menu of
  `account-menu.tsx`, where an owner with the Billing or the Viewer role
  claims the Admin role.
- `src/components/`: the shared parts, such as the dense `DataTable`, the
  figure strip in `kpis.tsx`, the status dots and tags in `status.tsx`, and the
  starts chart.
- `src/components/paths.ts`: every catalogue path of a screen and a flow.
- `src/components/dialog.tsx`: the dialog over a page, a sheet on mobile, such
  as Edit label, Clear cache, and Move to Free.
- `src/components/boat-fields.tsx`: the Boat key and the monthly spend limit,
  which every connection of Boat asks for, in setup and in Settings.
- `src/fixtures/sizes.ts`: the size of a label as text, the size of the
  default label, the least free disk of a label, the VM that ran a job with
  its VM disk, and the sizes that the Boat account can start, with their
  whole disks, which decide where the jobs of a label run.
- `src/fixtures/vm-disk.ts`: the VM disk of a job, which grows the root disk
  of the current image by the free disk of its label. It mirrors `RootSizing`
  of `ciproxy-agent`, and it holds the root size of the current image. No
  other mockup file repeats the rule or that size.
- `src/fixtures/hardware.ts`: the hardware of each machine, the slot rule,
  and the disk rule: the VM images keep 140 GB, the host keeps a quarter of
  the rest, each running job holds its VM disk, and the machine cache uses
  the rest.
- `src/fixtures/org.ts`: the workspace and its two GitHub organizations.
  `github-app.ts` holds the permissions of the GitHub App, and `access.ts` the
  sign-in length and the what-ifs of a suspended or removed App.
- `src/fixtures/org-links.ts`: the what-ifs of Add an organization: the Admin
  workspaces that the return lists, the rule that a new organization joins
  inactive on a full plan, and the holds of mokly-studio.
- `src/components/inactive-banner.tsx`: the banner of an inactive
  organization, and the reason of each hold, which its row repeats.
- `src/components/install-command.tsx`: the one-line install command, and the
  notices of a machine without KVM or with too little disk.
- `src/fixtures/site.ts`: the host of the public site. `cost-claim.ts` holds
  the cost claim of the landing page and its calculation, `plan-choice.ts` the
  cheapest plan that covers a use, and `plan-compare.ts` the feature
  comparison of the pricing page, from the tier features in `tiers.ts`.
- `src/fixtures/job-details.ts`: the jobs of the job page, with the label, the
  request that it gave the job or an unknown request, the Boat size that ran
  a job on Boat, and the cache source and counts.
- `src/fixtures/usage.ts`: Plan and usage at each moment, the included starts
  and own machines of a use, and the day on which the pace passes the
  included starts, which decides the pace notice of Free.
- `src/screens/billing/billing-scenes.tsx`: the billing pages of a moment,
  which its banner, its notices, and its links open.
- `src/fixtures/history-facts.ts`: the finished jobs of the History states
  of the API: a filtered list, a job that no runner took, a job with no
  finish time, and the date choices of the history window.
- `src/fixtures/job-details-facts.ts`: the job pages of facts that the API
  gives or lacks: a job that no runner took, a job with unknown facts, and
  the current-contract pages.
- `src/fixtures/wait-facts.ts`: the waiting jobs of a label whose fit is
  unknown and of a label that starts on the largest Boat size.
  `src/screens/jobs/live-fact-scenes.ts` and `live-fact-notices.tsx` build
  these moments of the live queue and their notices, and the moments of
  `place-facts` and `fit-facts`. `place-fact-notices.tsx` words the notices
  of a refused Boat size and of a label that no permitted place fits.
- `src/fixtures/savings-states.ts`: the figures of each Savings state,
  changed from the known report only where the state needs it: a cost that
  nobody set or that is known in part, a missing comparison price, a negative
  saving on a quiet day, a refresh, and ranges with no jobs.
- `src/fixtures/current-api.ts`: the offer of the API for each usage moment,
  and the moments that pass a limit, for the banners and the billing pages of
  the current API. `src/components/api-banner.tsx` words every banner as the
  API does, also the banner of a known moment.
- `src/screens/billing/api-notices.tsx`, `api-add-ons.tsx`, `api-review.tsx`,
  `api-outcome.tsx`, and `api-return.tsx`: the notices, the add-ons, the
  review, the unknown results, and the return of the billing pages of the
  current API.
- `src/components/state-schema.ts`: the closed prop schema of a screen whose
  states differ by one `state` prop.
- `src/screens/`: one component per screen, by area.
- `src/entries/`: the Mokly entry modules and `_folder.json` records. The
  directory tree is the catalogue tree.
- `scripts/check-policy.ts`: the policy check of `mockups:check`.
- `../../ts/mockups/package.json`: the dependencies and scripts of the
  mockups, with `eslint.config.js` beside it.

### Related Docs

- [The protocol docs](../protocol/) hold the requirements that the screens
  show, and the entries name them in `relatedDocs`.
- [The plan of the MVP foundation](../../plans/ciproxy-mvp-foundation.md)
  holds the decisions that the docs do not, such as the claims of the public
  site.
