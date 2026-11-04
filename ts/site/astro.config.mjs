import starlight from "@astrojs/starlight";
import { defineConfig } from "astro/config";

import { colors } from "@ciproxy/tokens";

/** A faint tint of a status token, for a marked line of a code block. */
const tint = (token, percent) =>
  `color-mix(in srgb, var(--c-status-${token}) ${percent}%, transparent)`;

export default defineConfig({
  site: "https://ciproxy.dev",
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
        "./src/styles/starlight/layout.css",
        "./src/styles/starlight/content.css",
        "./src/styles/starlight/blocks.css",
        "./src/styles/starlight/asides.css",
        "./src/styles/starlight/search.css",
        "./src/styles/starlight/search-results.css",
      ],
      // Code blocks use one light theme, also without JavaScript, with the
      // frame colours of the theme above. Starlight warns at each build that
      // one theme can lose contrast in dark mode, which this site never uses.
      // The rest gives them the look of the code blocks of the mockups: a
      // white head over the page grey, the token radius, code in the ink,
      // because colour marks status only, long lines that wrap, and diff
      // lines with a faint status tint and no edge of their own.
      expressiveCode: {
        themes: ["starlight-light"],
        useStarlightUiThemeColors: true,
        defaultProps: { wrap: true },
        styleOverrides: {
          borderRadius: "var(--r-lg)",
          borderColor: "var(--c-border)",
          codeFontSize: "0.78125rem",
          codeLineHeight: "1.25rem",
          codePaddingBlock: "0.625rem",
          codePaddingInline: "0.75rem",
          uiFontSize: "0.75rem",
          textMarkers: {
            insBackground: tint("ok", 12),
            insBorderColor: "transparent",
            delBackground: tint("err", 10),
            delBorderColor: "transparent",
          },
        },
        customizeTheme: (theme) => {
          for (const { scope, settings } of theme.settings) {
            if (!settings.foreground) continue;
            const comment = [scope ?? []].flat().join(" ").includes("comment");
            settings.foreground = comment ? colors.muted : colors.ink;
          }
          theme.fg = colors.ink;
          theme.colors["editor.foreground"] = colors.ink;
          theme.colors["editorGroupHeader.tabsBackground"] = "var(--c-surface)";
          theme.colors["titleBar.activeBackground"] = "var(--c-surface)";
          theme.colors["titleBar.activeForeground"] = "var(--c-muted)";
          theme.colors["tab.activeForeground"] = "var(--c-muted)";
          theme.colors["tab.activeBorderTop"] = "transparent";
          theme.styleOverrides.frames = {
            ...theme.styleOverrides.frames,
            editorActiveTabBackground: "var(--c-surface)",
            editorActiveTabIndicatorTopColor: "transparent",
            editorTabBarBorderBottomColor: "var(--c-border)",
            editorBackground: "var(--c-bg)",
            terminalTitlebarBorderBottomColor: "var(--c-border)",
            terminalTitlebarDotsOpacity: "0",
            terminalBackground: "var(--c-bg)",
          };
          theme.styleOverrides.textMarkers = {
            ...theme.styleOverrides.textMarkers,
            markBorderColor: "transparent",
          };
          return theme;
        },
      },
      sidebar: [
        { label: "Start here", items: [{ label: "Overview", link: "/docs/" }] },
      ],
    }),
  ],
});
