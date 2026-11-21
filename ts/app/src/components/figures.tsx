// The headline figures above a list: one box divided into cells by hairlines,
// each with a label and a value. A value is text, so an unknown figure can
// say so in place of a zero. The cells sit in one row on a wide window, and
// two by two on a phone, where an odd last cell takes the whole row.

import { StyleSheet, Text, View } from "react-native";

import { colors, fonts, radii } from "@ciproxy/tokens";

import { text } from "./text.ts";
import { useViewport } from "./viewport.ts";

export type Figure = { label: string; value: string };

function cellStyle(index: number, count: number, mobile: boolean) {
  if (!mobile) return [styles.cell, index > 0 && styles.left];
  const lastAlone = index === count - 1 && index % 2 === 0;
  return [
    styles.cell,
    lastAlone ? styles.full : styles.half,
    index % 2 === 1 && styles.left,
    index >= 2 && styles.top,
    styles.cellMobile,
  ];
}

export function FigureRow({ figures }: { figures: readonly Figure[] }) {
  const mobile = useViewport() === "mobile";
  return (
    <View style={[styles.row, mobile && styles.wrap]}>
      {figures.map((figure, index) => (
        <View
          key={figure.label}
          style={cellStyle(index, figures.length, mobile)}
        >
          <Text style={[text.small, styles.label]}>{figure.label}</Text>
          <Text
            style={[styles.value, mobile && styles.valueMobile]}
            numberOfLines={1}
          >
            {figure.value}
          </Text>
        </View>
      ))}
    </View>
  );
}

const styles = StyleSheet.create({
  row: {
    flexDirection: "row",
    overflow: "hidden",
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: radii.xl,
    backgroundColor: colors.surface,
  },
  wrap: { flexWrap: "wrap" },
  cell: {
    flexGrow: 1,
    flexBasis: 0,
    minWidth: 0,
    gap: 4,
    paddingTop: 16,
    paddingHorizontal: 20,
    paddingBottom: 18,
  },
  cellMobile: { paddingTop: 12, paddingHorizontal: 14, paddingBottom: 14 },
  half: { flexBasis: "50%", flexGrow: 0, width: "50%" },
  full: { flexBasis: "100%", width: "100%" },
  left: { borderLeftWidth: 1, borderLeftColor: colors.border },
  top: { borderTopWidth: 1, borderTopColor: colors.border },
  label: { color: colors.muted, lineHeight: 16 },
  value: {
    fontFamily: fonts.sans,
    fontSize: 26,
    fontWeight: "600",
    letterSpacing: -0.52,
    lineHeight: 32,
    color: colors.ink,
    fontVariant: ["tabular-nums"],
  },
  valueMobile: { fontSize: 24, lineHeight: 30 },
});
