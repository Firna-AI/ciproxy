// The ciproxy mark and wordmark, after the brand of the mockups. The mark is
// two bars that meet as a chevron: one flow of jobs, routed to an own machine
// or to overflow. The app runs on web only, so the mark is an inline SVG
// element, as Firna draws its icons on web.

import { Text } from "@firna/ui/typography";
import { StyleSheet, View } from "react-native";

import { colors } from "@ciproxy/tokens";

function BrandMark() {
  return (
    <svg
      aria-hidden="true"
      width={20}
      height={20}
      viewBox="0 0 24 24"
      fill={colors.ink}
    >
      <path d="M2 3h8.6l8.4 8.1h-8.6z" />
      <path d="M10.4 12.9H19L10.6 21H2z" />
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
