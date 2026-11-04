# @ciproxy/app

The private ciproxy dashboard, built with Expo Router and React Native Web.
It targets web only and has no open licence.
It displays no customer data and calls no backend yet.

The root route says what ciproxy does, in the Graphite look of the
signed-out screens of the [mockups](../../docs/mockups/README.md):
the ciproxy mark and wordmark above one card on the page grey.
It uses the Firna typography with the Graphite theme of the
[token package](../tokens/README.md).
The page is light only. It declares the `only light` colour scheme, so a
browser that darkens pages leaves it light.

## Development

Run from the repository root with Node 24.22.1:

```sh
npm --prefix ts ci
npm --prefix ts run app:start -- --host localhost
npm --prefix ts run app:typecheck
npm --prefix ts run app:lint
npm --prefix ts run app:export
npm --prefix ts --workspace @ciproxy/app run format
```

The development page is `/` on the URL printed by Expo.
The production export goes to `dist/`.
The package has no test runner because the static skeleton has no test logic.

Expo loads the shared Geist faces with `expo-font`.
The static export puts the faces in the page head, so the page renders at
once and the text takes the faces when they load.
Do not wait for the fonts before the first render.
The client would render nothing where the server rendered the page,
and React would report a hydration error.
The Metro configuration enables WOFF2 assets for these faces.

The app runs on web only, so the brand mark is an inline SVG element.

## Key Code

- [src/app/_layout.tsx](src/app/_layout.tsx) provides the shared Firna theme
  and loads the fonts.
- [src/app/index.tsx](src/app/index.tsx) is the dashboard home route.
- [src/components/brand.tsx](src/components/brand.tsx) is the ciproxy mark
  and wordmark.
- [app.json](app.json) limits Expo to web and enables static export.
- [tsconfig.json](tsconfig.json) enables strict TypeScript checks.

The [token package](../tokens/README.md) owns Graphite and the fonts.
The [gate](../../xtask/README.md) checks lint, types, format, and web export.
