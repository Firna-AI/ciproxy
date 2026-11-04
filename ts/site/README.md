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
- `/docs/` is Starlight with a Graphite theme. The docs pages come with
  their own work.

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
of the cheapest plan and the calculation of the cost claim.

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
- [src/content/docs/docs/index.md](src/content/docs/docs/index.md) is `/docs/`.
- [src/content/i18n/en.json](src/content/i18n/en.json) changes Starlight
  labels, such as the search label.
- [src/components/Tokens.astro](src/components/Tokens.astro) adds the token
  custom properties and the fonts to a page.

The [gate](../../xtask/README.md) checks the build, types, lint, and format.
