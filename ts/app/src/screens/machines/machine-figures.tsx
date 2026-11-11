// The figures of the Machines pages: the slots of a machine as a thin
// segmented bar with its legend, a meter against a limit, and the row of
// headline figures above the table, which becomes two by two on a phone.

import { StyleSheet, Text, View } from "react-native";
import { ProgressBar } from "@firna/ui/loader";

import { colors, fonts, radii, slotColors } from "@ciproxy/tokens";

import type { SlotState } from "../../api/schema.ts";
import { text } from "../../components/text.ts";
import { listCopy } from "../../view/machines-copy.ts";

/**
 * The corner of a slot mark: 1 px, as the mockups draw it. The token package
 * has no small-mark radius yet, so the value comes from the smallest radius
 * token, as the site derives its small marks.
 */
export const markRadius = radii.sm / 4;

/** The colour of each slot state, from the token package. */
export const slotFill: Readonly<Record<SlotState, string>> = {
  job: slotColors.busy,
  warm: slotColors.warm,
  free: slotColors.free,
};

/** One segment for each reported slot: a job, a warm runner, or free. */
export function SlotStrip({
  states,
  label,
}: {
  states: readonly SlotState[];
  /** The words of the bar for a screen reader, such as "5 busy · 1 warm". */
  label: string;
}) {
  return (
    <View style={styles.strip} role="img" aria-label={label}>
      {states.map((state, index) => (
        <View
          key={index}
          style={[styles.slot, { backgroundColor: slotFill[state] }]}
        />
      ))}
    </View>
  );
}

export function SlotLegend() {
  const items: readonly [SlotState, string][] = [
    ["job", listCopy.legend.job],
    ["warm", listCopy.legend.warm],
    ["free", listCopy.legend.free],
  ];
  return (
    <View style={styles.legend}>
      {items.map(([state, label]) => (
        <View key={state} style={styles.legendItem}>
          <View
            style={[styles.legendSlot, { backgroundColor: slotFill[state] }]}
          />
          <Text style={text.caption}>{label}</Text>
        </View>
      ))}
    </View>
  );
}

/** A ratio against a limit, with its figure beside the label. */
export function Meter({
  label,
  ratio,
  value,
  detail,
}: {
  label: string;
  ratio: number;
  value: string;
  detail?: string | null;
}) {
  return (
    <View style={styles.meter}>
      <View style={styles.meterHead}>
        <Text style={[text.small, text.strong]}>{label}</Text>
        <Text style={[text.small, styles.meterValue]}>{value}</Text>
      </View>
      <ProgressBar
        value={ratio}
        color={colors.primary}
        trackColor={colors.primarySoft}
        height={4}
        accessibilityLabel={label}
      />
      {detail ? <Text style={text.caption}>{detail}</Text> : null}
    </View>
  );
}

export type Figure = { label: string; value: string };

/** The headline figures: one row on a wide window, two by two on a phone. */
export function FigureRow({
  figures,
  compact,
}: {
  figures: readonly Figure[];
  compact: boolean;
}) {
  return (
    <View style={styles.figures}>
      {figures.map((figure, index) => (
        <View
          key={figure.label}
          style={[
            styles.figure,
            compact ? styles.figureCompact : styles.figureWide,
            index > 0 && !compact && styles.figureDivider,
            compact && index % 2 === 1 && styles.figureDivider,
            compact && index >= 2 && styles.figureTop,
          ]}
        >
          <Text style={styles.figureLabel}>{figure.label}</Text>
          <Text
            style={[styles.figureValue, compact && styles.figureValueCompact]}
          >
            {figure.value}
          </Text>
        </View>
      ))}
    </View>
  );
}

const styles = StyleSheet.create({
  strip: { flexDirection: "row", flexWrap: "wrap", gap: 2 },
  slot: { width: 14, height: 6, borderRadius: markRadius },
  legend: { flexDirection: "row", flexWrap: "wrap", columnGap: 16, rowGap: 6 },
  legendItem: { flexDirection: "row", alignItems: "center", gap: 6 },
  legendSlot: { width: 12, height: 4, borderRadius: markRadius },
  meter: { gap: 6 },
  meterHead: {
    flexDirection: "row",
    flexWrap: "wrap",
    justifyContent: "space-between",
    gap: 8,
  },
  meterValue: { fontVariant: ["tabular-nums"] },
  figures: {
    flexDirection: "row",
    flexWrap: "wrap",
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: radii.xl,
    backgroundColor: colors.surface,
    overflow: "hidden",
  },
  figure: { gap: 4, minWidth: 0 },
  figureWide: {
    flexGrow: 1,
    flexBasis: 0,
    paddingTop: 16,
    paddingBottom: 18,
    paddingHorizontal: 20,
  },
  figureCompact: {
    width: "50%",
    paddingTop: 12,
    paddingBottom: 14,
    paddingHorizontal: 14,
  },
  figureDivider: { borderLeftWidth: 1, borderLeftColor: colors.border },
  figureTop: { borderTopWidth: 1, borderTopColor: colors.border },
  figureLabel: {
    fontFamily: fonts.sans,
    fontSize: 13,
    lineHeight: 16,
    color: colors.muted,
  },
  figureValue: {
    fontFamily: fonts.sans,
    fontSize: 26,
    fontWeight: "600",
    letterSpacing: -0.5,
    lineHeight: 32,
    color: colors.ink,
    fontVariant: ["tabular-nums"],
  },
  figureValueCompact: { fontSize: 22, lineHeight: 28 },
});
