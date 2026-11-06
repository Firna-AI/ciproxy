import { SharedUiThemeProvider } from "@firna/ui/theme";
import { useFonts } from "expo-font";
import { Slot } from "expo-router";
import Head from "expo-router/head";

import { theme } from "@ciproxy/tokens/firna";
import { fontAssets } from "@ciproxy/tokens/font-assets";

import { ServicesProvider } from "../services/context.tsx";

export default function RootLayout() {
  // The static render puts the Geist faces in the page head, so the page
  // renders at once and the text takes the faces when they load. Waiting
  // for them here would render nothing on the client while the server
  // rendered the page, which breaks hydration.
  useFonts(fontAssets);

  return (
    <SharedUiThemeProvider theme={theme}>
      <Head>
        <title>ciproxy</title>
        {/* Graphite is light only, also when the browser darkens pages. */}
        <meta name="color-scheme" content="only light" />
        <link rel="icon" href="data:," />
      </Head>
      <ServicesProvider>
        <Slot />
      </ServicesProvider>
    </SharedUiThemeProvider>
  );
}
