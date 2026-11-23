import geist from "@fontsource-variable/geist/files/geist-latin-wght-normal.woff2";
import geistMono from "@fontsource-variable/geist-mono/files/geist-mono-latin-wght-normal.woff2";

import { fontNames } from "./index.ts";

/** Bundler asset IDs of the Geist faces, keyed by family name. */
export const fontAssets = {
  [fontNames.sans]: geist,
  [fontNames.mono]: geistMono,
};
