import starlight from "@astrojs/starlight";
import { defineConfig } from "astro/config";

export default defineConfig({
  integrations: [
    starlight({
      title: "ciproxy",
      components: {
        Head: "./src/components/starlight/Head.astro",
        Header: "./src/components/starlight/Header.astro",
        PageTitle: "./src/components/starlight/PageTitle.astro",
        ThemeProvider: "./src/components/starlight/ThemeProvider.astro",
        ThemeSelect: "./src/components/starlight/ThemeSelect.astro",
      },
      customCss: [
        "./src/styles/starlight/theme.css",
        "./src/styles/starlight/frame.css",
        "./src/styles/starlight/content.css",
        "./src/styles/starlight/asides.css",
      ],
      // Code blocks use one light theme, also without JavaScript, with the
      // frame colours of the theme above. Starlight warns at each build that
      // one theme can lose contrast in dark mode, which this site never uses.
      expressiveCode: {
        themes: ["starlight-light"],
        useStarlightUiThemeColors: true,
      },
      sidebar: [
        { label: "Start here", items: [{ label: "Overview", link: "/docs/" }] },
      ],
    }),
  ],
});
