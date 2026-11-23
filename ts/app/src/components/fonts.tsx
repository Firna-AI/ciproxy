// The Geist faces of Graphite. Expo's font loader writes each `@font-face`
// rule with no weight range, so a browser takes the variable file as weight
// 400 only. Safari then adds a synthetic bold on top of the real weight, and
// text at 500 or more shows too heavy. This component writes the rules with
// the full weight range in place of the font loader.
//
// The static render puts the rules and the preloads in the page head, so the
// page renders at once and the text takes the faces when they load.

import { Asset } from "expo-asset";
import Head from "expo-router/head";

import { fontFaceCss } from "@ciproxy/tokens/css";
import { fontAssets } from "@ciproxy/tokens/font-assets";

const fontUrls = Object.fromEntries(
  Object.entries(fontAssets).map(([family, asset]) => [
    family,
    Asset.fromModule(asset).uri,
  ]),
);

const fontCss = fontFaceCss(fontUrls);

export function GraphiteFonts() {
  return (
    <Head>
      {Object.values(fontUrls).map((url) => (
        <link
          key={url}
          rel="preload"
          href={url}
          as="font"
          type="font/woff2"
          crossOrigin=""
        />
      ))}
      <style>{fontCss}</style>
    </Head>
  );
}
