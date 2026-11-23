// The CPU and memory panel of the job page, after the mockups. It takes the
// full width under the two panels. A finished job shows its average CPU and
// its peak memory against the VM that ran it, its out-of-memory kills on
// Linux, a chart over time with each step marked, and the figures of each
// step, with a note that each figure covers the whole VM. When the VM is
// larger than the size of its label, each meter marks the label size with a
// tick over its bar. A running job and a job without figures show one
// sentence in place of the figures.

import { StyleSheet, Text, View } from "react-native";
import { ProgressBar } from "@firna/ui/loader";
import { SkeletonBar, SkeletonGroup } from "@firna/ui/skeleton";
import { StatusDot } from "@firna/ui/status-dot";

import {
  chartColors,
  colors,
  fonts,
  markRadii,
  statusColors,
} from "@ciproxy/tokens";

import { Panel } from "../../components/panel.tsx";
import { text } from "../../components/text.ts";
import { useViewport } from "../../components/viewport.ts";
import {
  usageCopy,
  type UsageFigure,
  type UsageMeter,
  type UsagePanelModel,
} from "../../view/jobs-usage.ts";
import { UsageChart } from "./usage-chart.tsx";
import { UsageSteps } from "./usage-steps.tsx";

type MeasuredPanel = Extract<UsagePanelModel, { state: "measured" }>;

/** A meter against the VM, with the label size as a tick over the bar. */
function Meter({ label, meter }: { label: string; meter: UsageMeter }) {
  return (
    <View style={styles.meter}>
      <ProgressBar
        value={meter.share}
        color={colors.primary}
        trackColor={colors.primarySoft}
        height={4}
        accessibilityLabel={label}
      />
      {meter.mark === null ? null : (
        <View
          aria-hidden
          style={[styles.mark, { left: `${meter.mark * 100}%` }]}
        />
      )}
    </View>
  );
}

function Figure({ figure }: { figure: UsageFigure | null }) {
  if (figure === null) return <View style={styles.figure} />;
  return (
    <View style={styles.figure}>
      <Text style={styles.label}>{figure.label}</Text>
      <View style={styles.value}>
        <View style={styles.number}>
          {figure.alert ? (
            <StatusDot color={statusColors.err} size="sm" />
          ) : null}
          <Text style={styles.numberText}>{figure.value}</Text>
        </View>
        {figure.caption === null ? null : (
          <Text style={text.caption}>{figure.caption}</Text>
        )}
      </View>
      {figure.meter === null ? null : (
        <Meter label={figure.label} meter={figure.meter} />
      )}
    </View>
  );
}

/** The figures: three on a row, or two on a row on a phone. */
function Figures({
  figures,
  mobile,
}: {
  figures: readonly UsageFigure[];
  mobile: boolean;
}) {
  const columns = mobile ? 2 : 3;
  const rows: (UsageFigure | null)[][] = [];
  for (let start = 0; start < figures.length; start += columns) {
    const row: (UsageFigure | null)[] = figures.slice(start, start + columns);
    while (row.length < columns) row.push(null);
    rows.push(row);
  }
  return (
    <View style={mobile ? styles.figuresMobile : styles.figures}>
      {rows.map((row, index) => (
        <View
          key={index}
          style={[styles.figureRow, mobile && styles.figureRowMobile]}
        >
          {row.map((figure, column) => (
            <Figure key={figure?.label ?? column} figure={figure} />
          ))}
        </View>
      ))}
    </View>
  );
}

function Measured({ model }: { model: MeasuredPanel }) {
  const mobile = useViewport() === "mobile";
  const note = [styles.note, mobile && styles.noteMobile];
  return (
    <Panel title={usageCopy.title} flush>
      <View style={[styles.body, mobile && styles.bodyMobile]}>
        <Figures figures={model.figures} mobile={mobile} />
        <UsageChart usage={model.usage} mobile={mobile} />
      </View>
      {model.steps === null ? (
        <View style={note}>
          <Text style={[text.small, styles.muted]}>{usageCopy.noSteps}</Text>
        </View>
      ) : (
        <>
          <UsageSteps
            steps={model.steps}
            killColumn={model.killColumn}
            mobile={mobile}
          />
          <View style={note}>
            <Text style={text.caption}>{usageCopy.wholeVm}</Text>
          </View>
        </>
      )}
    </Panel>
  );
}

/** The panel of a job: its figures, or one sentence in their place. */
export function UsagePanel({ model }: { model: UsagePanelModel }) {
  if (model.state === "measured") return <Measured model={model} />;
  return (
    <Panel title={usageCopy.title}>
      <Text style={[text.small, styles.muted]}>{model.text}</Text>
    </Panel>
  );
}

/** The panel while the job loads. */
export function UsageLoading() {
  return (
    <Panel title={usageCopy.title}>
      <SkeletonGroup direction="column" gap={12}>
        <SkeletonBar width="40%" height={14} />
        <SkeletonBar width="100%" height={120} />
        <SkeletonBar width="90%" height={14} />
        <SkeletonBar width="90%" height={14} />
      </SkeletonGroup>
    </Panel>
  );
}

const styles = StyleSheet.create({
  body: { gap: 20, padding: 16 },
  bodyMobile: { padding: 14 },
  figures: { gap: 16 },
  figuresMobile: { gap: 14 },
  figureRow: { flexDirection: "row", gap: 24 },
  figureRowMobile: { gap: 16 },
  figure: { flexGrow: 1, flexShrink: 1, flexBasis: 0, minWidth: 0, gap: 6 },
  label: {
    fontFamily: fonts.sans,
    fontSize: 13,
    lineHeight: 16,
    color: colors.muted,
  },
  value: {
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "baseline",
    columnGap: 6,
  },
  number: { flexDirection: "row", alignItems: "center", gap: 8 },
  numberText: {
    fontFamily: fonts.sans,
    fontSize: 20,
    fontWeight: "600",
    letterSpacing: -0.2,
    lineHeight: 26,
    color: colors.ink,
  },
  meter: { position: "relative" },
  // The tick of the label size, ringed in the surface colour so it shows on
  // the fill and on the track.
  mark: {
    position: "absolute",
    top: -3,
    width: 2,
    height: 10,
    marginLeft: -1,
    borderRadius: markRadii.sm,
    backgroundColor: chartColors.context,
    boxShadow: `0 0 0 1px ${colors.surface}`,
  },
  note: {
    paddingTop: 10,
    paddingHorizontal: 16,
    paddingBottom: 12,
    borderTopWidth: 1,
    borderTopColor: colors.border,
  },
  noteMobile: { paddingHorizontal: 14 },
  muted: { color: colors.muted },
});
