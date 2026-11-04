# @ciproxy/site

The private ciproxy marketing and documentation site.
One Astro project serves the landing page at `/` and Starlight at `/docs/`.
The site has no open licence.

Both parts have the Graphite look of the mockups under
[docs/mockups](../../docs/mockups/README.md):

- One top bar serves both parts. The landing page shows the wordmark and the
  main links. The docs show the wordmark with "Docs" and the search box.
- The landing page shows the hero of the landing mockup.
- The docs use a Graphite theme of Starlight. Black is the one accent.
  Geist and Geist Mono are the faces.
- The site is light only and has no theme switch. It stays light when the
  system or a stored choice asks for dark, and also without JavaScript.
- An aside (`:::note`, `:::tip`, `:::caution`, `:::danger`) is a box with a
  full hairline border, never a left accent edge. A note or a tip is white,
  a caution is amber, and a danger is rose.
- A docs page shows its `description` as the lead under the title.

Rules for work on the site:

- Take every colour, font, and radius from the `--c-*`, `--r-*`, and
  `--font-*` custom properties of the [token package](../tokens/README.md).
- Add a link to the top bar or a page only when its page exists. Pricing,
  Sign in, Start free, and the GitHub link of the mockups wait for their
  pages. `mainLinks` in the top bar holds the main links.

## Development

Run from the repository root with Node 24.22.1:

```sh
npm --prefix ts ci
npm --prefix ts run site:dev -- --host 0.0.0.0
npm --prefix ts run site:typecheck
npm --prefix ts run site:lint
npm --prefix ts run site:build
npm --prefix ts --workspace @ciproxy/site run format
```

Open `/` and `/docs/` on the URL printed by Astro.
The production build goes to `dist/`.
The typecheck uses `astro check` with the strict TypeScript configuration.
ESLint uses the Astro plugin and recommended TypeScript rules.
Prettier uses the Astro plugin.

The build prints a Starlight warning about one code theme.
That warning is about dark mode, which the site does not use.
Astro keeps rendered pages in `node_modules/.astro`.
Delete that folder after a change to the code block settings.

The site declares the YAML version that Starlight needs for server rendering.
This keeps it separate from the Markdown lint tool's YAML version.

## Key Code

- [astro.config.mjs](astro.config.mjs) registers Starlight, its component
  overrides, the theme files, and the sidebar.
- [src/pages/index.astro](src/pages/index.astro) is the landing page.
- [src/layouts/MarketingLayout.astro](src/layouts/MarketingLayout.astro) is
  the frame of a marketing page.
- [src/components/site-header/SiteHeader.astro](src/components/site-header/SiteHeader.astro)
  is the top bar. Its `section` prop selects the landing or the docs form.
- [src/components/starlight/](src/components/starlight/Header.astro) holds
  the Starlight overrides: the head, the top bar with the search, the page
  title with its lead, and the light-only theme provider and theme switch.
- [src/styles/starlight/theme.css](src/styles/starlight/theme.css) maps the
  Starlight variables to the tokens. The other files in that folder restyle
  the frame, the article, and the asides.
- [src/content/docs/docs/index.md](src/content/docs/docs/index.md) is `/docs/`.
- [src/content/i18n/en.json](src/content/i18n/en.json) changes Starlight
  labels, such as the search label.
- [src/components/Tokens.astro](src/components/Tokens.astro) adds the token
  custom properties and the fonts to a page.

The [gate](../../xtask/README.md) checks the build, types, lint, and format.
