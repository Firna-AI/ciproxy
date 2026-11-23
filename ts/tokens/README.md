# @ciproxy/tokens

The shared Graphite values for the mockups, dashboard, and public site.
The package is private and has no open licence.

## Entries

- `@ciproxy/tokens` exposes plain colours, fonts, radii, and colour groups.
  It has no runtime dependency.
- `radii` shapes controls and boxes, and the Firna theme reads it.
  `markRadii` holds the 1, 2, and 3 px corners of small data marks, such as
  slot segments, chart keys, and the data ends of thin bars.
- `@ciproxy/tokens/firna` creates the Firna theme.
  It uses the peer dependency `@firna/ui`.
- `@ciproxy/tokens/fonts` loads Geist and Geist Mono through CSS imports.
- `@ciproxy/tokens/font-assets` exposes the same faces as bundler assets for
  the dashboard. Expo does not resolve the local font URLs in the CSS imports.
- `@ciproxy/tokens/css` exposes `tokenDeclarations()` and `tokenCss`.
  These define the `--c-*`, `--r-*`, and `--font-*` custom properties.
- `fontFaceCss()` in the CSS entry writes the `@font-face` rules for the
  font asset URLs. Each rule declares the weight range `100 900` of the
  variable files. Without it, a browser takes each face as weight 400 only,
  and Safari adds a synthetic bold on top of the real weight. Do not load the
  faces through Expo's font loader, because it writes no weight range.
  The mark radii take the `--r-mark-*` names, such as `--r-mark-sm`.
  Firna consumers add their focus ring properties at the Firna boundary.

The root-level TypeScript entries also support the mockup path mapping.
Keep values here so every consumer reads the same source.
The mockups and the site use the same declaration builder from the CSS entry.

## Development

Use the Node version in [.nvmrc](../../.nvmrc).
Run these commands from the repository root:

```sh
npm --prefix ts ci
npm --prefix ts run tokens:check
npm --prefix ts run tokens:test
npm --workspace @ciproxy/tokens --prefix ts run format
```

The unit test checks CSS name conversion with Node's built-in test runner.
The [gate](../../xtask/README.md) also checks format and file length.
