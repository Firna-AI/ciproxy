// A status: a small dot beside its word, never colour alone and never a pill.
// The dot takes its colour from the status tokens. A dot without a word, such
// as at the start of a list row, names the status for screen readers.

import { StyleSheet, Text, View } from "react-native";
import { StatusDot } from "@firna/ui/status-dot";

import { colors, statusColors, type StatusTone } from "@ciproxy/tokens";

import { text } from "./text.ts";

export type { StatusTone };

/** The dot alone, with the status as its accessible name. */
export function StatusMark({
  tone,
  label,
}: {
  tone: StatusTone;
  label: string;
}) {
  return <StatusDot color={statusColors[tone]} size="sm" label={label} />;
}

/** The dot beside its word. */
export function StatusLabel({
  tone,
  children,
}: {
  tone: StatusTone;
  children: string;
}) {
  return (
    <View style={styles.row}>
      <StatusDot color={statusColors[tone]} size="sm" />
      <Text style={[text.small, styles.word]} numberOfLines={1}>
        {children}
      </Text>
    </View>
  );
}

const styles = StyleSheet.create({
  row: { flexDirection: "row", alignItems: "center", gap: 8, flexShrink: 0 },
  word: { color: colors.ink2 },
});
