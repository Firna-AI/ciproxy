# Mockups

The mockups of ciproxy: the hosted dashboard, and the public site with the
marketing pages and the docs. They are part of the spec: each screen of the MVP
renders in a mobile and a desktop variant, with the empty, loading, and error
states that it needs, and the user flows step through those screens.
[Mokly](https://github.com/mokly-ai/mokly) builds the catalogue from React entry
modules.

## Commands

Run these from the repository root. Mokly needs Node 22.15 or newer, but not
Node 23 and not Node 24.14 to 24.18.

```sh
npm --prefix ts ci                   # install the workspace
npm --prefix ts run mockups:build    # strict build into docs/mockups/mokly-generated
npm --prefix ts run mockups:check    # build, typecheck, lint, format check, policy check
npm --prefix ts run mockups:serve    # browse with live reload and Changes against origin/main
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
- Fixture data lives in `src/fixtures/`. It tells one story: mokly-ai signs up
  on 1 September 2026, passes the Free starts on 4 September, moves to Team
  that day, and runs three Hetzner AX42 machines with Boat overflow from
  mid-September. The November screens show the busy Team organization, and the
  December screens the payment problems. Each tier counts only the starts of
  its own period, which begins on the day the tier begins.
- The customer's own hardware is a "machine" on every screen, also in the
  pricing: never "node" or "server".
- The public site has two sections: Marketing, with the landing page and the
  pricing page, and Docs, in the structure of Starlight: the sidebar of
  sections, the top bar with search, the article, and "On this page". Both
  share the site top bar, and `ciproxy.dev` stands in for the domain. The
  landing page shows real numbers from the fixtures and no customer logos,
  quotes, or press.
- One value has one format: `src/fixtures/format.ts` holds the money, count,
  and date formats, and `unbroken()` keeps a short value such as a date on one
  line.
- The look is Graphite: light and monochrome, Geist and Geist Mono, hairline
  borders, small radii, and black primary buttons. Colour marks status only,
  as a dot beside its word. Its tokens live in `src/theme/graphite.ts`, and a
  stylesheet reads them as custom properties, never as colour literals.
- Check `@firna/ui` and `src/components/` before you build a control. The Firna
  DOM build renders without React Native; the renderer adds its static CSS and
  the Graphite theme.
- A banner of a passed limit or a failed payment comes from
  `src/components/usage-banner.tsx`, so one state shows one banner on every
  page.
- Never edit or commit `docs/mockups/mokly-generated/`. Change the sources and
  build again.

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
- `src/theme/graphite.ts`: the Graphite tokens: the Firna theme with the Geist
  faces, and the status, chart, slot, and sidebar colours.
- `src/theme/firna.tsx`: publishes the tokens as CSS custom properties, such as
  `--c-ink`, `--c-status-ok`, `--r-md`, and `--font-mono`, beside the static
  Firna CSS.
- `src/components/site/site-header.tsx`: the top bar of the public site,
  shared by the marketing pages and the docs, with the search box on the docs.
- `src/screens/marketing/marketing-shell.tsx`: the frame of a marketing page:
  the sections between two hairline rails, and the footer. `section.tsx` holds
  the section head and the cells divided by hairlines.
- `src/screens/docs/docs-shell.tsx`: the frame of a docs page, in the structure
  of Starlight, with the sidebar from `docs-nav.ts` and the article parts in
  `docs-parts.tsx`.
- `src/components/shell/app-shell.tsx`: the signed-in frame: a white sidebar
  and a top bar with the organization and the page as breadcrumbs on desktop,
  and a top bar with tabs on mobile, with the banner slot for soft-limit and
  payment warnings.
- `src/components/`: the shared parts, such as the dense `DataTable`, the
  figure strip in `kpis.tsx`, the status dots and tags in `status.tsx`, and the
  starts chart.
- `src/components/paths.ts`: every catalogue path of a screen and a flow.
- `src/fixtures/site.ts`: the placeholder host of the public site and the
  worked example of the landing page; `plan-compare.ts` holds the feature
  comparison of the pricing page, from the tiers in `tiers.ts`.
- `src/components/state-schema.ts`: the closed prop schema of a screen whose
  states differ by one `state` prop.
- `src/screens/`: one component per screen, by area.
- `src/entries/`: the Mokly entry modules and `_folder.json` records. The
  directory tree is the catalogue tree.
- `scripts/check-policy.ts`: the policy check of `mockups:check`.
- `../../ts/mockups/package.json`: the dependencies and scripts of the
  mockups, with `eslint.config.js` beside it.

### Related Docs

- [The plan of the MVP foundation](../../plans/ciproxy-mvp-foundation.md)
  holds the decisions that the screens show. The entries name it in
  `relatedDocs` until the protocol docs exist.
