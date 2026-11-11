// The Boat spend of the month against the limit: the costs that Boat
// confirmed and the costs that it has not confirmed yet, each with its own
// figure, and a bar that stacks both against the limit. The warning tone
// follows the API flag of a reached limit. Settings under Overflow shows it
// with its explanation; the Machines page can show it compact.

import { StyleSheet, Text, View } from "react-native";

import { colors, radii } from "@ciproxy/tokens";

import { text } from "../../components/text.ts";
import type { SpendFigures } from "../../view/boat.ts";

function width(share: number): `${number}%` {
  return `${Math.round(share * 1000) / 10}%`;
}

function Part({
  label,
  value,
  confirmed,
  warning,
}: {
  label: string;
  value: string;
  confirmed: boolean;
  warning: boolean;
}) {
  return (
    <View style={styles.part} role="listitem">
      <View style={styles.partLabel}>
        <View
          style={[
            styles.key,
            confirmed
              ? warning
                ? styles.warningFill
                : styles.confirmedFill
              : styles.unconfirmedFill,
          ]}
        />
        <Text style={text.small}>{label}</Text>
      </View>
      <Text style={[text.small, styles.value]}>{value}</Text>
    </View>
  );
}

export function SpendFiguresView({
  figures,
  compact = false,
}: {
  figures: SpendFigures;
  /** A row of a page, with no explanation below. */
  compact?: boolean;
}) {
  const warning = figures.reached;
  return (
    <View style={styles.spend}>
      <View style={styles.spread}>
        <Text style={[text.small, text.strong]}>Spent this month</Text>
        <Text style={text.small}>Limit {figures.limit}</Text>
      </View>
      <View
        style={[styles.bar, warning && styles.barWarning]}
        role="img"
        aria-label={`${figures.confirmed} confirmed and ${figures.unconfirmed} not yet confirmed, of the ${figures.limit} limit`}
      >
        <View
          style={[
            warning ? styles.warningFill : styles.confirmedFill,
            { width: width(figures.confirmedShare) },
          ]}
        />
        <View
          style={[
            styles.unconfirmedFill,
            { width: width(figures.unconfirmedShare) },
          ]}
        />
      </View>
      <View role="list" style={styles.parts}>
        <Part
          label="Confirmed by Boat"
          value={figures.confirmed}
          confirmed
          warning={warning}
        />
        <Part
          label="Not yet confirmed"
          value={figures.unconfirmed}
          confirmed={false}
          warning={warning}
        />
      </View>
      {compact ? null : (
        <Text style={text.caption}>
          Not yet confirmed counts each start at its highest cost until Boat
          reports the cost.
          {figures.settled === null
            ? ""
            : ` It includes ${figures.settled} for starts that Boat did not report before their lifetime ended.`}{" "}
          At the limit, jobs run on your machines only.
        </Text>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  spend: { gap: 8 },
  spread: {
    flexDirection: "row",
    justifyContent: "space-between",
    gap: 12,
  },
  bar: {
    flexDirection: "row",
    height: 4,
    overflow: "hidden",
    borderRadius: radii.pill,
    backgroundColor: colors.primarySoft,
  },
  barWarning: { backgroundColor: colors.amberSoft },
  confirmedFill: { backgroundColor: colors.primary },
  warningFill: { backgroundColor: colors.amber },
  unconfirmedFill: { backgroundColor: colors.faint },
  parts: { gap: 4 },
  part: {
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "space-between",
    gap: 12,
  },
  partLabel: { flexDirection: "row", alignItems: "center", gap: 6 },
  key: { width: 8, height: 8, borderRadius: radii.sm },
  value: { color: colors.ink, fontVariant: ["tabular-nums"] },
});
