# Mockups

The mockups of the ciproxy dashboard, the hosted web product. They are part of
the spec: each screen of the MVP renders in a mobile and a desktop variant, with
the empty, loading, and error states that it needs, and the user flows step
through those screens. [Mokly](https://github.com/mokly-ai/mokly) builds the
catalogue from React entry modules.

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
  on 1 November 2026, passes the Free starts on 4 November, moves to Team, and
  runs three Hetzner AX42 servers with Boat overflow by mid-November.
- Check `@firna/ui` before you build a control. Its DOM build renders without
  React Native; the renderer adds its static CSS and theme.
- A design direction is a theme layer, not a copy of a screen. A
  `DirectionScope` around a screen sets the tokens of the direction, and the
  shared components read `useDirection()` to draw it. Outside a scope, every
  component keeps the current look.
- A direction rule in CSS repeats the default selector under `.dir`, or
  `.dir-<key>` for one direction, in a `*-dir.css` file that the component
  imports after its default stylesheet.
- Never edit or commit `docs/mockups/mokly-generated/`. Change the sources and
  build again.

## Visual smoke test

1. Run `npm --prefix ts run mockups:check`.
2. Open each changed screen at
   `docs/mockups/mokly-generated/static/mokly-generated/<path>/index.mobile.html`
   and `index.desktop.html`. Check the desktop document at 1180 px and at
   720 px: the catalogue shows the desktop frame at about 720 px beside the
   phone, where the sidebar becomes an icon rail. Check the mobile document at
   366 px, the width of the phone screen.
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
- `src/render.tsx`: the renderer. It adds the Inter face, the base stylesheet,
  the static Firna CSS, and the Firna theme provider around every screen.
- `src/theme/firna.tsx`: the Firna theme, the chart colours, and the tokens as
  CSS custom properties for the stylesheets.
- `src/theme/direction.ts`: the `Direction` token set, `useDirection()`, and
  `useRoutes()`, which the shared components read.
- `src/theme/direction-scope.tsx`: `DirectionScope`, which publishes the
  tokens of a direction on its class and nests its Firna theme.
- `src/theme/directions/`: the three directions, Graphite, Midnight, and
  Enterprise, with their fonts.
- `src/screens/directions/direction-screens.tsx`: the key screens of one
  direction, for the entries under `src/entries/design-directions/`.
- `src/components/shell/app-shell.tsx`: the signed-in frame, a sidebar on
  desktop and a top bar with tabs on mobile, with the banner slot for soft-limit
  and payment warnings.
- `src/components/paths.ts`: every catalogue path of a screen and a flow.
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
