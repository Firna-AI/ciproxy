// The steps of a job with their figures: a table on a wider window and a
// dense list on a phone, in the order of GitHub, from Set up job to Complete
// job. Each step leads with its number on GitHub. A name that hides a script
// links to that step on GitHub, where its log names what it ran. A long name
// wraps inside the Step column, also within a word. A skipped step, and a
// step that holds no point, show no figures. The out-of-memory column shows
// only when a step ran out of memory.

import { useMemo, type ReactNode } from "react";
import { StyleSheet, Text, View, type ViewStyle } from "react-native";
import { createTableStyles, type TableColumn } from "@firna/ui/table";
import { useSharedUiTheme } from "@firna/ui/theme";

import { colors, fonts } from "@ciproxy/tokens";

import { StatusLabel } from "../../components/status-label.tsx";
import { text } from "../../components/text.ts";
import { TextLink } from "../../components/text-link.tsx";
import {
  figureLine,
  usageCopy,
  type UsageStepRow,
} from "../../view/jobs-usage.ts";

type Size = "body" | "small";

const columns: TableColumn[] = [
  { key: "step", label: "Step", flex: 1 },
  { key: "duration", label: "Duration", width: 88, align: "right" },
  { key: "cpu", label: "Average CPU", width: 112, align: "right" },
  { key: "memory", label: "Peak memory", width: 112, align: "right" },
];

const killColumn: TableColumn = {
  key: "kills",
  label: "Out of memory",
  width: 128,
  align: "right",
};

/**
 * The number, then the name in one slot that can shrink below its longest
 * word, as the mockup's `.step-name` does, so a long word breaks inside the
 * Step column. A name that hides a script links to the step on GitHub, in a
 * new tab.
 */
function StepTitle({ step, size }: { step: UsageStepRow; size: Size }) {
  return (
    <View style={styles.title}>
      <Text style={styles.number}>{step.number}</Text>
      <View style={styles.name} testID="usage-step-name">
        {step.href === null ? (
          <Text style={[size === "body" ? text.body : text.small, styles.ink]}>
            {step.name}
          </Text>
        ) : (
          <TextLink
            label={step.name}
            href={step.href}
            size={size}
            underlined
            external
            title={`Open step ${step.number} on GitHub`}
            accessibilityLabel={`${step.name}, opens step ${step.number} on GitHub`}
          />
        )}
      </View>
    </View>
  );
}

/** The duration; in a row of the list it takes the line of the name. */
function Duration({ step, size }: { step: UsageStepRow; size: Size }) {
  const words = size === "body" && !step.timed ? text.body : text.small;
  return (
    <Text
      style={[
        words,
        step.timed ? styles.mono : styles.muted,
        size === "body" && styles.rowDuration,
      ]}
    >
      {step.duration}
    </Text>
  );
}

/** A figure that a step does not have: a dash, and words for a reader. */
function NoFigure() {
  return (
    <View>
      <Text aria-hidden style={[text.small, styles.muted]}>
        —
      </Text>
      <Text style={styles.hidden}>{usageCopy.noFigures}</Text>
    </View>
  );
}

function cell(step: UsageStepRow, key: string): ReactNode {
  if (key === "step") return <StepTitle step={step} size="small" />;
  if (key === "duration") return <Duration step={step} size="small" />;
  const figures = step.figures;
  if (figures === null) return <NoFigure />;
  if (key === "cpu")
    return <Text style={[text.small, styles.figure]}>{figures.cpu}</Text>;
  if (key === "memory")
    return <Text style={[text.small, styles.figure]}>{figures.memory}</Text>;
  return figures.kills === null ? null : (
    <StatusLabel tone="err">{figures.kills}</StatusLabel>
  );
}

function StepRows({ steps }: { steps: readonly UsageStepRow[] }) {
  return (
    <View style={styles.rows} role="list" aria-label="Steps">
      {steps.map((step, index) => (
        <View
          key={step.number}
          role="listitem"
          style={[styles.row, index > 0 && styles.divided]}
        >
          <View style={styles.main}>
            <StepTitle step={step} size="body" />
            {step.figures === null ? null : (
              <Text style={[text.caption, styles.indent]}>
                {figureLine(step.figures)}
              </Text>
            )}
            {step.figures === null || step.figures.kills === null ? null : (
              <View style={styles.indent}>
                <StatusLabel tone="err">
                  {`Out of memory: ${step.figures.kills}`}
                </StatusLabel>
              </View>
            )}
          </View>
          <Duration step={step} size="body" />
        </View>
      ))}
    </View>
  );
}

/** A fixed width wins; otherwise the column shares the rest by its flex. */
function columnStyle(column: TableColumn): ViewStyle {
  return column.width === undefined
    ? { flex: column.flex ?? 1, minWidth: 0 }
    : { width: column.width };
}

/**
 * The steps on a wider window, in the look of the Firna table. That table
 * gives a screen reader no roles, so this one names the table, its rows, its
 * column headers, and its cells.
 */
function StepTable({
  steps,
  columns: shown,
}: {
  steps: readonly UsageStepRow[];
  columns: readonly TableColumn[];
}) {
  const theme = useSharedUiTheme();
  const table = useMemo(() => createTableStyles(theme), [theme]);
  const box = (column: TableColumn) => [
    columnStyle(column),
    column.align === "right" && table.cellRight,
  ];
  return (
    <View role="table" aria-label="Steps" style={table.table}>
      <View role="row" style={table.headRow}>
        {shown.map((column) => (
          <View key={column.key} role="columnheader" style={box(column)}>
            <Text style={[table.th, column.align === "right" && table.thRight]}>
              {column.label}
            </Text>
          </View>
        ))}
      </View>
      {steps.map((step, index) => (
        <View
          key={step.number}
          role="row"
          style={[table.row, index === steps.length - 1 && table.rowLast]}
        >
          {shown.map((column) => (
            <View key={column.key} role="cell" style={box(column)}>
              {cell(step, column.key)}
            </View>
          ))}
        </View>
      ))}
    </View>
  );
}

export function UsageSteps({
  steps,
  killColumn: killed,
  mobile,
}: {
  steps: readonly UsageStepRow[];
  /** A step ran out of memory, so the table shows that column. */
  killColumn: boolean;
  mobile: boolean;
}) {
  if (mobile) return <StepRows steps={steps} />;
  return (
    <View style={styles.table}>
      <StepTable
        steps={steps}
        columns={killed ? [...columns, killColumn] : columns}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  table: { borderTopWidth: 1, borderTopColor: colors.border },
  title: {
    flexDirection: "row",
    alignItems: "baseline",
    gap: 10,
    flexShrink: 1,
    minWidth: 0,
  },
  number: {
    minWidth: 16,
    flexShrink: 0,
    fontFamily: fonts.sans,
    fontSize: 12,
    lineHeight: 16,
    textAlign: "right",
    color: colors.muted,
    fontVariant: ["tabular-nums"],
  },
  // The slot of the name shrinks to the Step column, and the name wraps in
  // it, also within a word. A bare Text in the row would keep the width of
  // its longest word and run past the column.
  name: { flexShrink: 1, minWidth: 0 },
  ink: { color: colors.ink },
  mono: { fontFamily: fonts.mono, color: colors.ink },
  muted: { color: colors.muted },
  rowDuration: { flexShrink: 0, lineHeight: 20 },
  figure: { color: colors.ink, fontVariant: ["tabular-nums"] },
  // Words for a screen reader only, as the Firna controls hide them.
  hidden: {
    position: "absolute",
    width: 1,
    height: 1,
    margin: -1,
    overflow: "hidden",
    opacity: 0,
  },
  rows: { borderTopWidth: 1, borderTopColor: colors.border },
  row: {
    flexDirection: "row",
    alignItems: "flex-start",
    justifyContent: "space-between",
    gap: 12,
    paddingVertical: 10,
    paddingHorizontal: 14,
  },
  divided: { borderTopWidth: 1, borderTopColor: colors.border },
  main: { flexShrink: 1, minWidth: 0, gap: 2 },
  // The figures under a name line up with the name, after the number.
  indent: { paddingLeft: 26 },
});
