// The text styles of the Graphite screens, after the mockups. Colours come
// from the token package; the sizes are the type scale of the mockups.

import { StyleSheet } from "react-native";

import { colors, fonts } from "@ciproxy/tokens";

export const text = StyleSheet.create({
  title: {
    fontFamily: fonts.sans,
    fontSize: 22,
    fontWeight: "600",
    letterSpacing: -0.44,
    lineHeight: 28,
    color: colors.ink,
  },
  pageTitle: {
    fontFamily: fonts.sans,
    fontSize: 20,
    fontWeight: "600",
    letterSpacing: -0.4,
    lineHeight: 26,
    color: colors.ink,
  },
  heading: {
    fontFamily: fonts.sans,
    fontSize: 15,
    fontWeight: "600",
    lineHeight: 20,
    color: colors.ink,
  },
  lead: {
    fontFamily: fonts.sans,
    fontSize: 14,
    lineHeight: 20,
    color: colors.ink2,
  },
  body: {
    fontFamily: fonts.sans,
    fontSize: 14,
    lineHeight: 20,
    color: colors.ink,
  },
  small: {
    fontFamily: fonts.sans,
    fontSize: 13,
    lineHeight: 18,
    color: colors.ink2,
  },
  caption: {
    fontFamily: fonts.sans,
    fontSize: 12,
    lineHeight: 16,
    color: colors.muted,
  },
  strong: { fontWeight: "500", color: colors.ink },
  mono: { fontFamily: fonts.mono },
});
