# @ciproxy/site

The private ciproxy marketing and documentation site at `ciproxy.dev`.
One Astro project serves the marketing pages and Starlight at `/docs/`.
The site has no open licence.

The pages follow the marketing and docs mockups under
[docs/mockups](../../docs/mockups/README.md):

- `/` is the landing page: the hero with a picture of the product, how it
  works, the features, the cost claim, the pricing teaser, security, and the
  closing call to action.
- `/pricing/` has the tiers, the add-ons, the feature comparison, and the
  questions. The last question shows the calculation of the cost claim.
- `/terms/` and `/privacy/` are the frames of the legal pages. Grey lines
  hold the place of the text that the owner writes or approves before the
  launch. Only the data retention of the privacy policy has text.
- `/docs/` is Starlight with a Graphite theme. Its sidebar has the 14 pages
  of the docs sidebar mockup, in its order:
  - Start here: Overview at `/docs/`, Quickstart, and How ciproxy works.
  - Guides under `/docs/guides/`: Add a machine, Connect Boat, Move your
    workflows, Set a spend limit, and Remove a machine.
  - Reference under `/docs/reference/`: Labels and VM sizes, Agent install
    options, GitHub App permissions, and Plans and starts.
  - Security under `/docs/security/`: Job isolation, and The agent
    connection.

The look is Graphite:

- One top bar serves both parts. The marketing pages show the wordmark,
  Pricing, Docs, Sign in, and Start free. On a phone, a menu holds the links.
  The docs show the wordmark with "Docs", the Starlight search, and the same
  actions.
- Every marketing page ends with the footer.
- The docs use a Graphite theme of Starlight. Black is the one accent, and
  Geist and Geist Mono are the faces. Code is in the ink, because colour
  marks status only.
- The site is light only and has no theme switch. It stays light when the
  system or a stored choice asks for dark, and also without JavaScript.
- An aside (`:::note`, `:::tip`, `:::caution`, `:::danger`) is a box with a
  full hairline border, never a left accent edge. A note or a tip is white,
  a caution is amber, and a danger is rose. No other box has a left accent
  edge either.
- The search dialog is the Starlight search with Pagefind, restyled with CSS
  only.

Rules for work on the site:

- Take every colour, font, and radius from the `--c-*`, `--r-*`, and
  `--font-*` custom properties of the [token package](../tokens/README.md).
- Take every price, limit, and tier feature from `src/data/tiers.ts`. A
  feature that ships after the launch says "Coming later".
- The cost claim computes every number from the dated list prices in
  `src/data/price-lists.ts`. Check those prices and their date each month.
- Every button that opens sign-in says "Start free". It and the "Sign in"
  link open the dashboard at `https://app.ciproxy.dev/sign-in`.
- Add a link only when its page exists. A marketing page links to a docs
  page through `docsLinks`, which shows the link only once the page exists.
  The GitHub link of the mockups waits for the public repository of the
  agent.
- The picture of the product in the hero is a static illustration with
  fixed values, and its caption says so. The site shows no customer logos,
  quotes, or press.
- Icons are the Lucide paths in `src/components/icon/icons.ts`, under the
  licence beside them. The site adds no icon package.

Rules for the docs pages:

- A docs page states only what the [protocol docs](../../docs/protocol/)
  require and the product does. The dashboard steps use the names of the
  protocol docs, such as **Machines**, **Add machine**, and **Settings**
  under **Overflow**.
- The Markdown check of the gate refuses HTML and component tags, such as
  `<Steps>` or `<LinkCard>`, in the source of a docs page. An MDX page shows
  a Starlight component without a tag, in one of two ways:
  - It maps a Markdown element to a component with
    `export const components`. The Quickstart maps its ordered list to
    `StepsList`, which puts the list in Starlight `Steps`.
  - It calls `jsx()` of `astro/jsx-runtime` in an expression, such as
    `{jsx(Badge, { text: "Default" })}`. The Overview and the Quickstart
    draw their link cards with `CardGrid` and `LinkCard` in this way.
- Keep the data of such a call in an `export const` block, and the call on
  one line. Prettier formats the block as JavaScript, but it formats an
  expression on several lines as Markdown text and drops its indent.
- A page that shows a value of `src/data/` is MDX. It imports the value and
  writes it as an expression, such as `{freePlanNote}`. Prices, limits, and
  tier features come from `src/data/tiers.ts`. The VM disk of a default job
  and the image budget come from `src/data/hardware.ts`, because they change
  with the image.
- A code example that shows a change, but copies the code after it, uses
  `CodeBlock` through `jsx()`, with its distinct `copy` text. The Quickstart
  and Move your workflows show the runs-on change with `runsOnJob` of
  `src/data/workflow.ts`, and Copy gives the whole job after the change.
  `CodeBlock` carries `not-content`, so the Starlight article styles stay
  out of it.
- A link between docs pages uses its absolute address, such as
  `/docs/reference/labels/`, so that the gate does not read it as a file.
- One fact has one page, and the other pages link to it. The full example
  of a job container is under "Run steps in another image" of Move your
  workflows. The list of supported Linux distributions is under "What the
  machine needs" of Add a machine.

## Development

Use the Node version in [.nvmrc](../../.nvmrc).
Run these commands from the repository root:

```sh
npm --prefix ts ci
npm --prefix ts run site:dev -- --host 0.0.0.0
npm --prefix ts run site:typecheck
npm --prefix ts run site:lint
npm --prefix ts --workspace @ciproxy/site run test
npm --prefix ts run site:build
npm --prefix ts --workspace @ciproxy/site run format
```

Open `/` and `/docs/` on the URL printed by Astro.
The production build goes to `dist/`.
The typecheck uses `astro check` with the strict TypeScript configuration.
ESLint uses the Astro plugin and recommended TypeScript rules.
Prettier uses the Astro plugin.
The tests under `_tests_` run on the Node test runner. They cover the choice
of the cheapest plan, the calculation of the cost claim, and the Copy text of
the runs-on examples of the docs. That last test builds the site with
`astro build` into a temporary folder and reads the Copy text of each example
from the built pages. `_tests_/built-site.ts`
holds that build and the reader of the Copy text.
The claim keeps hardware slots separate from concurrent default jobs.
It subtracts the 140 GB image budget before the host and job disk split.
Each default job reserves its VM disk: the current image with the 20 GB of
free disk of the label `ciproxy`. `src/data/hardware.ts` holds that VM disk
as one value from `RootSizing` of `ciproxy-agent`. Change the value when the
image or that rule changes. The docs page Labels and VM sizes shows it in
whole GB. The AX42 calculation combines its two 512 GB
disks into one 1,024 GB filesystem, which fits all six slots.

The build prints a Starlight warning about one code theme.
That warning is about dark mode, which the site does not use.
The build also says that no docs entry `404` exists, so Starlight uses its
own 404 page.
Astro keeps rendered pages in `node_modules/.astro`.
Delete that folder after a change to the code block settings.

The site declares the YAML version that Starlight needs for server rendering.
This keeps it separate from the Markdown lint tool's YAML version.

## Key Code

- [astro.config.mjs](astro.config.mjs) registers Starlight, its component
  overrides, the theme files, the code block look, and the sidebar.
- [src/pages/](src/pages/index.astro) holds the marketing pages: the landing
  page, the pricing page, and the two legal pages.
- [src/layouts/MarketingLayout.astro](src/layouts/MarketingLayout.astro) is
  the frame of a marketing page: the top bar, the rails, and the footer.
- [src/components/marketing/](src/components/marketing/Section.astro) holds
  the sections of the landing page, the section parts, the code block, and
  the footer.
- [src/components/pricing/](src/components/pricing/PriceCards.astro) holds
  the parts of the pricing page.
- [src/components/legal/LegalPage.astro](src/components/legal/LegalPage.astro)
  is the frame of the terms and the privacy policy.
- [src/components/site-header/SiteHeader.astro](src/components/site-header/SiteHeader.astro)
  is the top bar. Its `section` prop selects the page that it marks, and the
  docs form.
- [src/components/LinkButton.astro](src/components/LinkButton.astro) is a
  link with the look of a button, and
  [src/components/icon/Icon.astro](src/components/icon/Icon.astro) an icon.
- [src/data/](src/data/tiers.ts) holds the data of the pages: the tiers, the
  plan choice, the comparison, the price lists, the slot rule, the cost
  claim, the legal sections, the addresses, and the values of the
  illustration.
- [src/components/starlight/](src/components/starlight/Header.astro) holds
  the Starlight overrides: the head, the top bar with the search, the page
  title with its lead, and the light-only theme provider and theme switch.
- [src/styles/starlight/theme.css](src/styles/starlight/theme.css) maps the
  Starlight variables to the tokens. The other files in that folder restyle
  the frame, the columns, the article, its blocks, the asides, and the
  search.
- [src/content/docs/docs/](src/content/docs/docs/index.mdx) holds the docs
  pages: `index.mdx` is `/docs/`, and the folders `guides`, `reference`, and
  `security` hold the pages of those sidebar groups.
- [src/components/docs/StepsList.astro](src/components/docs/StepsList.astro)
  shows the ordered list of a docs page as Starlight steps.
- [src/data/docs-links.ts](src/data/docs-links.ts) names the docs pages that
  the marketing pages link to, such as the docs links of the footer.
- [src/content/i18n/en.json](src/content/i18n/en.json) changes Starlight
  labels, such as the search label.
- [src/components/Tokens.astro](src/components/Tokens.astro) adds the token
  custom properties and the fonts to a page.

The [gate](../../xtask/README.md) checks the build, types, lint, and format.
