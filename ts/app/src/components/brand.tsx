// The ciproxy mark and wordmark, after the brand of the mockups. The mark is
// two bars that meet as a chevron: one flow of jobs, routed to an own machine
// or to overflow. The app runs on web only, so the mark is an inline SVG
// element, as Firna draws its icons on web. The favicon draws the same paths.

import { Text } from "@firna/ui/typography";
import { StyleSheet, View } from "react-native";

import { colors } from "@ciproxy/tokens";

/** The paths of the mark, in a 24 by 24 box. */
const markPaths = [
  "M2 3h8.6l8.4 8.1h-8.6z",
  "M10.4 12.9H19L10.6 21H2z",
] as const;

/**
 * The favicon as a data URL: the mark in the ink, with no tile behind it. Its
 * square box fits the mark with half a unit of space, so the mark fills a
 * small tab.
 */
export const brandFaviconUrl = `data:image/svg+xml,${encodeURIComponent(
  `<svg xmlns="http://www.w3.org/2000/svg" viewBox="1 2.5 19 19" ` +
    `fill="${colors.ink}">` +
    markPaths.map((d) => `<path d="${d}"/>`).join("") +
    `</svg>`,
)}`;

export function BrandMark({ size = 20 }: { size?: number }) {
  return (
    <svg
      aria-hidden="true"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill={colors.ink}
    >
      {markPaths.map((d) => (
        <path key={d} d={d} />
      ))}
    </svg>
  );
}

export function Brand() {
  return (
    <View style={styles.brand}>
      <BrandMark />
      <Text variant="label" style={styles.word}>
        ciproxy
      </Text>
    </View>
  );
}

const styles = StyleSheet.create({
  brand: { flexDirection: "row", alignItems: "center", gap: 8 },
  word: { fontSize: 15, letterSpacing: -0.375, lineHeight: 20 },
});
