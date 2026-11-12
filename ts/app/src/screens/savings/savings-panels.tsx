// The panels of the savings report: the savings figure with the cost bars
// and the price-list note, and the cost table. The ciproxy bar takes the
// accent of the chart tokens, the alternatives their context grey, and each
// bar names its cost in words beside it.

import { StyleSheet, Text, View } from "react-native";
import { Table, type TableColumn } from "@firna/ui/table";

import { chartColors, colors, fonts, radii } from "@ciproxy/tokens";

import { Panel } from "../../components/panel.tsx";
import { text } from "../../components/text.ts";
import { useViewport } from "../../components/viewport.ts";
import type { SavingsPage } from "../../api/savings-types.ts";
import { costRows, type CostRow } from "../../view/savings-costs.ts";
import {
  costBars,
  priceListNote,
  savingsHero,
} from "../../view/savings-report.ts";

export function ReportPanel({ page }: { page: SavingsPage }) {
  const mobile = useViewport() === "mobile";
  const hero = savingsHero(page);
  return (
    <Panel>
      {hero ? (
        <View style={styles.hero}>
          <Text style={styles.figure}>{hero.figure}</Text>
          <Text style={[text.small, styles.heroText]}>{hero.text}</Text>
        </View>
      ) : null}
      <View style={styles.bars} role="list" aria-label="Cost of the same jobs">
        {costBars(page).map((bar) => (
          <View
            key={bar.key}
            style={[styles.barRow, mobile && styles.barRowMobile]}
            role="listitem"
            aria-label={`${bar.label}: ${bar.value}`}
          >
            <Text
              style={[
                text.small,
                styles.barLabel,
                bar.accent && text.strong,
                !mobile && styles.barLabelWide,
              ]}
            >
              {bar.label}
            </Text>
            <View style={styles.track}>
              <View
                style={[
                  styles.bar,
                  {
                    width: `${Math.max(bar.share * 80, 0.5)}%`,
                    backgroundColor: bar.accent
                      ? chartColors.accent
                      : chartColors.context,
                  },
                ]}
              />
              <Text style={[text.small, styles.barValue]}>{bar.value}</Text>
            </View>
          </View>
        ))}
      </View>
      <Text style={[text.caption, styles.note]}>{priceListNote(page)}</Text>
    </Panel>
  );
}

const columns: TableColumn[] = [
  { key: "label", label: "", flex: 3 },
  { key: "jobs", label: "Jobs", flex: 2, align: "right" },
  { key: "minutes", label: "Minutes", flex: 2, align: "right" },
  { key: "cost", label: "Cost", flex: 2, align: "right" },
];

function cell(row: CostRow, key: string) {
  const value =
    key === "jobs"
      ? row.jobs
      : key === "minutes"
        ? row.minutes
        : key === "cost"
          ? row.cost
          : row.label;
  return (
    <Text style={[text.small, styles.cell, row.total && styles.total]}>
      {value}
    </Text>
  );
}

export function CostPanel({ page }: { page: SavingsPage }) {
  return (
    <Panel title="Your cost with ciproxy" flush>
      <Table
        columns={columns}
        rows={costRows(page)}
        rowKey={(row) => row.key}
        cell={cell}
        accessibilityLabel="Your cost with ciproxy"
      />
    </Panel>
  );
}

const styles = StyleSheet.create({
  hero: { gap: 6, marginBottom: 20 },
  figure: {
    fontFamily: fonts.sans,
    fontSize: 28,
    fontWeight: "600",
    letterSpacing: -0.56,
    lineHeight: 34,
    color: colors.ink,
    fontVariant: ["tabular-nums"],
  },
  heroText: { maxWidth: 560 },
  bars: { gap: 10 },
  barRow: { flexDirection: "row", alignItems: "center", gap: 16 },
  barRowMobile: { flexDirection: "column", alignItems: "stretch", gap: 6 },
  barLabel: { color: colors.ink2 },
  barLabelWide: { width: 120, flexShrink: 0 },
  track: { flexDirection: "row", alignItems: "center", gap: 8, flexGrow: 1 },
  bar: {
    height: 10,
    borderTopRightRadius: radii.sm,
    borderBottomRightRadius: radii.sm,
  },
  barValue: { color: colors.ink, fontWeight: "500" },
  note: { marginTop: 14 },
  cell: { color: colors.ink, fontVariant: ["tabular-nums"] },
  total: { fontWeight: "600" },
});
