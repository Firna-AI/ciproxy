import { SharedUiThemeProvider } from "@firna/ui/theme";
import { Slot } from "expo-router";
import Head from "expo-router/head";

import { theme } from "@ciproxy/tokens/firna";

import { GraphiteFonts } from "../components/fonts.tsx";
import { ServicesProvider } from "../services/context.tsx";

export default function RootLayout() {
  return (
    <SharedUiThemeProvider theme={theme}>
      <Head>
        <title>ciproxy</title>
        {/* Graphite is light only, also when the browser darkens pages. */}
        <meta name="color-scheme" content="only light" />
        <link rel="icon" href="data:," />
      </Head>
      <GraphiteFonts />
      <ServicesProvider>
        <Slot />
      </ServicesProvider>
    </SharedUiThemeProvider>
  );
}
