// Settings under Overflow while its settings load: the panel of the switch
// and the wait, and one provider panel, with grey bars in place of their
// content, as the loading state of the Settings mockups shows, so the page
// keeps its layout when the settings arrive. The providers are not known
// yet, so the provider panel has no title.

import { StyleSheet, View } from "react-native";

import { colors, radii } from "@ciproxy/tokens";

import { Panel } from "../../components/panel.tsx";

const titles = ["When jobs start on a provider", undefined] as const;

export function OverflowLoading() {
  return (
    <View
      style={styles.page}
      role="status"
      aria-label="Loading Overflow"
      aria-busy
    >
      {titles.map((title, index) => (
        <Panel key={title ?? index} title={title}>
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
