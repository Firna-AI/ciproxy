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
  the disk rule, in which each running job holds the disk of its size.
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
- "Size" is what a label gives a job: its vCPU, memory, and disk. Write "VM
  size" where the word needs a qualifier. Never write "shape" or "machine
  size". Code names use the same words, such as `sizeText` and `LabelDialog`.

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
  default label, and the sizes that the Boat account can start, which decide
  where the jobs of a label run.
- `src/fixtures/hardware.ts`: the hardware of each machine, the slot rule,
  and the disk rule: the VM images keep 140 GB, the host keeps a quarter of
  the rest, each running job holds the disk of its size, and the machine cache
  uses the rest.
- `src/fixtures/org.ts`: the workspace and its two GitHub organizations.
  `github-app.ts` holds the permissions of the GitHub App, and `access.ts` the
  sign-in length and the what-ifs of a suspended or removed App.
- `src/components/install-command.tsx`: the one-line install command, and the
  notices of a machine without KVM or with too little disk.
- `src/fixtures/site.ts`: the host of the public site. `cost-claim.ts` holds
  the cost claim of the landing page and its calculation, `plan-choice.ts` the
  cheapest plan that covers a use, and `plan-compare.ts` the feature
  comparison of the pricing page, from the tier features in `tiers.ts`.
- `src/fixtures/job-details.ts`: the jobs of the job page, with the label, the
  Boat size that ran a job on Boat, and the cache source and counts.
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
