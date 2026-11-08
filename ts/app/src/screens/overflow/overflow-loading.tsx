// Settings under Overflow while its settings load: its three panels with
// grey bars in place of their content, as the loading state of the Settings
// mockups shows, so the page keeps its layout when the settings arrive.

import { StyleSheet, View } from "react-native";

import { colors, radii } from "@ciproxy/tokens";

import { Panel } from "../../components/panel.tsx";

const titles = [
  "Boat",
  "When jobs start on Boat",
  "Monthly spend limit",
] as const;

export function OverflowLoading() {
  return (
    <View
      style={styles.page}
      role="status"
      aria-label="Loading Overflow"
      aria-busy
    >
      {titles.map((title) => (
        <Panel key={title} title={title}>
          <View style={styles.bars}>
            <View style={[styles.bar, styles.long]} />
            <View style={[styles.bar, styles.short]} />
          </View>
        </Panel>
      ))}
    </View>
  );
}

const styles = StyleSheet.create({
  page: { gap: 20 },
  bars: { gap: 8 },
  bar: { height: 8, borderRadius: radii.sm, backgroundColor: colors.bg2 },
  long: { width: "60%" },
  short: { width: "32%" },
});
