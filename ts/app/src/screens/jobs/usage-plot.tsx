// One plot of the CPU and memory chart: its title with its unit, its value
// axis, and the plot itself. The top of the plot is the vCPU or the memory of
// the VM. The plot draws its line, the wash under it, the start of each step,
// and the size of the label as SVG in a box of the seconds of the chart
// across and 0 to 100 down, which stretches to the width of the panel; each
// stroke keeps its width as the box stretches. A dot ringed in the surface
// colour marks each out-of-memory kill. The rows of the chart share one
// column for the axis labels, so the plots line up.

import type { CSSProperties } from "react";
import { StyleSheet, Text, View } from "react-native";

import {
  chartColors,
  colors,
  fonts,
  radii,
  statusColors,
} from "@ciproxy/tokens";

import type {
  ChartPlot,
  UsageChartModel,
} from "../../view/jobs-usage-chart.ts";

const svgStyle: CSSProperties = {
  position: "absolute",
  top: 0,
  left: 0,
  width: "100%",
  height: "100%",
  overflow: "visible",
};

export function percent(share: number): `${number}%` {
  return `${share * 100}%`;
}

export function Plot({
  plot,
  model,
  first,
  height,
}: {
  plot: ChartPlot;
  model: UsageChartModel;
  /** The plot starts the chart, as without a lane of steps. */
  first: boolean;
  height: number;
}) {
  return (
    <>
      <View style={chartRows.row}>
        <View style={chartRows.axis} />
        <Text style={[styles.title, first && styles.firstTitle]}>
          {plot.title} <Text style={styles.unit}>in {plot.unit}</Text>
        </Text>
      </View>
      <View style={chartRows.row} aria-hidden>
        <View style={[chartRows.axis, { height }]}>
          {plot.ticks.map((tick, index) => (
            <Text
              key={index}
              style={[
                chartRows.axisText,
                styles.yTick,
                { bottom: percent(tick.share) },
              ]}
            >
              {tick.text}
            </Text>
          ))}
        </View>
        <View style={[styles.plot, { height }]}>
          {plot.ticks.map((tick, index) => (
            <View
              key={index}
              style={[
                styles.grid,
                tick.share === 0 && styles.base,
                { bottom: percent(tick.share) },
              ]}
            />
          ))}
          <svg
            viewBox={`0 0 ${model.seconds} 100`}
            preserveAspectRatio="none"
            aria-hidden="true"
            style={svgStyle}
          >
            {model.starts.map((start) => (
              <line
                key={start}
                x1={start}
                x2={start}
                y1={0}
                y2={100}
                stroke={colors.border}
                strokeWidth={1}
                vectorEffect="non-scaling-stroke"
              />
            ))}
            <path d={plot.area} fill={chartColors.own} fillOpacity={0.1} />
            {plot.label === null ? null : (
              <line
                x1={0}
                x2={model.seconds}
                y1={plot.label}
                y2={plot.label}
                stroke={chartColors.context}
                strokeWidth={1}
                vectorEffect="non-scaling-stroke"
              />
            )}
            <path
              d={plot.line}
              fill="none"
              stroke={chartColors.own}
              strokeWidth={1.5}
              strokeLinecap="round"
              strokeLinejoin="round"
              vectorEffect="non-scaling-stroke"
            />
          </svg>
          {plot.kills.map((kill, index) => (
            <View
              key={index}
              style={[
                styles.kill,
                { left: percent(kill.left), top: percent(kill.top) },
              ]}
            />
          ))}
        </View>
      </View>
    </>
  );
}

/** The rows of the chart: a column of axis labels, then the content. */
export const chartRows = StyleSheet.create({
  row: { flexDirection: "row", columnGap: 10 },
  axis: { position: "relative", width: 30, flexShrink: 0 },
  axisText: {
    fontFamily: fonts.mono,
    fontSize: 11,
    lineHeight: 12,
    color: colors.muted,
  },
});

const styles = StyleSheet.create({
  title: {
    flexGrow: 1,
    flexShrink: 1,
    marginTop: 12,
    marginBottom: 8,
    fontFamily: fonts.sans,
    fontSize: 12,
    fontWeight: "500",
    lineHeight: 16,
    color: colors.ink2,
  },
  firstTitle: { marginTop: 0 },
  unit: { fontWeight: "400", color: colors.muted },
  yTick: { position: "absolute", right: 0, transform: [{ translateY: "50%" }] },
  plot: { position: "relative", flexGrow: 1, flexShrink: 1 },
  grid: {
    position: "absolute",
    left: 0,
    right: 0,
    height: 0,
    borderTopWidth: 1,
    borderTopColor: colors.border,
  },
  base: { borderTopColor: colors.border2 },
  kill: {
    position: "absolute",
    width: 8,
    height: 8,
    borderRadius: radii.pill,
    backgroundColor: statusColors.err,
    boxShadow: `0 0 0 2px ${colors.surface}`,
    transform: [{ translateX: "-50%" }, { translateY: "-50%" }],
  },
});
