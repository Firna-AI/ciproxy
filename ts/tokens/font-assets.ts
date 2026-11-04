import geist from "@fontsource-variable/geist/files/geist-latin-wght-normal.woff2";
import geistMono from "@fontsource-variable/geist-mono/files/geist-mono-latin-wght-normal.woff2";

import { fontNames } from "./index.ts";

/** Bundler asset IDs for Expo's font loader, from the same Geist packages. */
export const fontAssets = {
  [fontNames.sans]: geist,
  [fontNames.mono]: geistMono,
};
