// The CPU and memory of a job over time, as the mockup draws it: a lane of
// the steps, two plots on one time axis, and a key. @firna/ui/chart cannot
// mark steps, and it draws at a fixed width in a static render, so this
// chart draws each plot as SVG in a box that stretches to the panel. A
// number names each step block that is wide enough for it. Each colour comes
// from the token package: the plots take the series colour of the chart
// tokens, and the label size takes the context grey. The figure names the
// figures of the job for screen readers, and hides the lane and the axes
// from them. The time axis measures its width, so a label that the panel
// would cut off moves just inside the part of the panel that shows. The key
// names the label line and the out-of-memory dot.

import { useState } from "react";
import { StyleSheet, Text, View } from "react-native";

import {
  chartColors,
  colors,
  fonts,
  markRadii,
  radii,
  statusColors,
} from "@ciproxy/tokens";

import type { MeasuredUsage } from "../../view/jobs-usage.ts";
import {
  usageChartModel,
  type LaneBlock,
  type UsageChartModel,
} from "../../view/jobs-usage-chart.ts";
import type { TimeAxis, TimeTick } from "../../view/jobs-usage-times.ts";
import { axisColumn, chartRows, percent, Plot } from "./usage-plot.tsx";

/** The narrowest step block, in pixels, that shows its number. */
const numberWidth = 22;

function Lane({ blocks }: { blocks: readonly LaneBlock[] }) {
  const [width, setWidth] = useState(0);
  return (
    <View style={chartRows.row} aria-hidden>
      <View style={[chartRows.axis, styles.laneLabel]}>
        <Text style={chartRows.axisText}>Step</Text>
      </View>
      <View
        style={styles.lane}
        onLayout={(event) => setWidth(event.nativeEvent.layout.width)}
      >
        {blocks.map((block) => (
          <View
            key={block.number}
            style={[
              styles.step,
              { left: percent(block.left), width: percent(block.width) },
            ]}
          >
            <View style={styles.block} testID="usage-mark-step">
              <Text style={styles.blockText}>
                {block.width * width >= numberWidth ? block.number : ""}
              </Text>
            </View>
          </View>
        ))}
      </View>
    </View>
  );
}

/** A label centred on its time, or at the visible edge that it would pass. */
function timePlace(tick: TimeTick, axis: TimeAxis) {
  if (tick.align === "start") return { left: -axis.left };
  if (tick.align === "end") return { right: -axis.right };
  return [styles.timeCenter, { left: percent(tick.share) }];
}

function Keys({ model }: { model: UsageChartModel }) {
  if (model.labelKey === null && !model.killKey) return null;
  return (
    <View style={[chartRows.row, styles.keysRow]}>
      <View style={chartRows.axis} />
      <View style={styles.keys}>
        {model.labelKey === null ? null : (
          <View style={styles.key}>
            <View style={styles.keyLabel} aria-hidden />
            <Text style={styles.keyText}>{model.labelKey}</Text>
          </View>
        )}
        {model.killKey ? (
          <View style={styles.key}>
            <View style={styles.keyKill} aria-hidden />
            <Text style={styles.keyText}>Out of memory</Text>
          </View>
        ) : null}
      </View>
    </View>
  );
}

export function UsageChart({
  usage,
  mobile,
  room,
}: {
  usage: MeasuredUsage;
  mobile: boolean;
  /** The room in pixels that the panel shows past each side of the chart. */
  room: number;
}) {
  const [width, setWidth] = useState(0);
  // Left of the time axis, the panel also shows the column of axis labels.
  const axis = {
    width,
    left: axisColumn.width + axisColumn.gap + room,
    right: room,
  };
  const model = usageChartModel(usage, mobile, axis);
  const height = mobile ? 64 : 76;
  return (
    <View role="figure" aria-label={model.summary}>
      {model.lane === null ? null : <Lane blocks={model.lane} />}
      {model.plots.map((plot, index) => (
        <Plot
          key={plot.title}
          plot={plot}
          model={model}
          first={index === 0 && model.lane === null}
          height={height}
        />
      ))}
      <View style={[chartRows.row, styles.timesRow]} aria-hidden>
        <View style={chartRows.axis} />
        <View
          style={styles.times}
          onLayout={(event) => setWidth(event.nativeEvent.layout.width)}
        >
          {model.times.map((tick) => (
            <Text
              key={tick.text}
              testID="usage-time"
              style={[chartRows.axisText, styles.xTick, timePlace(tick, axis)]}
            >
              {tick.text}
            </Text>
          ))}
        </View>
      </View>
      <Keys model={model} />
    </View>
  );
}

const styles = StyleSheet.create({
  laneLabel: { justifyContent: "center", alignItems: "flex-end" },
  lane: { position: "relative", flexGrow: 1, flexShrink: 1, height: 18 },
  step: { position: "absolute", top: 0, bottom: 0, paddingHorizontal: 1 },
  block: {
    flexGrow: 1,
    alignItems: "center",
    justifyContent: "center",
    borderRadius: markRadii.md,
    backgroundColor: colors.bg2,
  },
  blockText: {
    fontFamily: fonts.mono,
    fontSize: 11,
    lineHeight: 12,
    color: colors.ink2,
  },
  timesRow: { marginTop: 8 },
  times: { position: "relative", flexGrow: 1, flexShrink: 1, height: 12 },
  xTick: { position: "absolute" },
  timeCenter: { transform: [{ translateX: "-50%" }] },
  keysRow: { marginTop: 14 },
  keys: {
    flexGrow: 1,
    flexShrink: 1,
    flexDirection: "row",
    flexWrap: "wrap",
    rowGap: 6,
    columnGap: 16,
  },
  key: { flexDirection: "row", alignItems: "center", gap: 6 },
  keyText: {
    fontFamily: fonts.sans,
    fontSize: 12,
    lineHeight: 16,
    color: colors.ink2,
  },
  keyLabel: {
    width: 14,
    height: 0,
    borderTopWidth: 2,
    borderTopColor: chartColors.context,
  },
  keyKill: {
    width: 8,
    height: 8,
    borderRadius: radii.pill,
    backgroundColor: statusColors.err,
  },
});
