// A use against its limit: the label, the figure, a thin bar, and a line
// under it. A warning turns the bar amber; the words say why.

import { StyleSheet, Text, View } from "react-native";
import { ProgressBar } from "@firna/ui/loader";

import { colors } from "@ciproxy/tokens";

import type { MeterModel } from "../../view/billing-usage.ts";
import { text } from "../text.ts";

export function Meter({ model }: { model: MeterModel }) {
  const { label, value, ratio, warning, detail } = model;
  return (
    <View style={styles.meter}>
      <View style={styles.head}>
        <Text style={[text.body, text.strong]}>{label}</Text>
        <Text style={[text.body, styles.value]}>{value}</Text>
      </View>
      <ProgressBar
        value={ratio}
        color={warning ? colors.amber : colors.primary}
        trackColor={warning ? colors.amberSoft : colors.primarySoft}
        height={4}
        accessibilityLabel={`${label}: ${value}`}
      />
      {detail ? <Text style={text.caption}>{detail}</Text> : null}
    </View>
  );
}

/** A figure with no limit to fill against, in the layout of a meter. */
export function MeterLine({
  label,
  value,
  detail,
}: {
  label: string;
  value: string;
  detail: string;
}) {
  return (
    <View style={styles.meter}>
      <View style={styles.head}>
        <Text style={[text.body, text.strong]}>{label}</Text>
        <Text style={[text.body, styles.value]}>{value}</Text>
      </View>
      <Text style={text.caption}>{detail}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  meter: { gap: 6 },
  head: {
    flexDirection: "row",
    flexWrap: "wrap",
    justifyContent: "space-between",
    columnGap: 12,
  },
  value: { fontVariant: ["tabular-nums"] },
});
