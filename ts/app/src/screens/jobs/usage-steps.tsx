// The steps of a job with their figures: a table on a wider window and a
// dense list on a phone, in the order of GitHub, from Set up job to Complete
// job. Each step leads with its number on GitHub. A name that hides a script
// links to that step on GitHub, where its log names what it ran. A skipped
// step, and a step that holds no point, show no figures. The out-of-memory
// column shows only when a step ran out of memory.

import { useState, type ReactNode } from "react";
import { StyleSheet, Text, View } from "react-native";
import { useFocusRing } from "@firna/ui/focusRing";
import { Table, type TableColumn } from "@firna/ui/table";

import { colors, fonts } from "@ciproxy/tokens";

import { ExternalIcon } from "../../components/icons.tsx";
import { StatusLabel } from "../../components/status-label.tsx";
import { text } from "../../components/text.ts";
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

const linkSizes = {
  body: { fontSize: 14, lineHeight: "20px" },
  small: { fontSize: 13, lineHeight: "18px" },
} as const;

/** A name that hides a script: a link to the step on GitHub, in a new tab. */
function StepLink({
  step,
  href,
  size,
}: {
  step: UsageStepRow;
  href: string;
  size: Size;
}) {
  const focus = useFocusRing();
  const [hovered, setHovered] = useState(false);
  return (
    <a
      href={href}
      target="_blank"
      rel="noopener noreferrer"
      title={`Open step ${step.number} on GitHub`}
      aria-label={`${step.name}, opens step ${step.number} on GitHub`}
      {...focus.focusRingDomProps}
      onFocus={focus.onFocus}
      onBlur={focus.onBlur}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: 4,
        color: hovered ? colors.ink : colors.ink2,
        fontFamily: fonts.sans,
        ...linkSizes[size],
        fontWeight: 500,
        overflowWrap: "anywhere",
        ...focus.focusRingVariables,
      }}
    >
      <span
        style={{
          textDecorationLine: "underline",
          textDecorationColor: hovered ? "currentColor" : colors.border2,
          textUnderlineOffset: "3px",
        }}
      >
        {step.name}
      </span>
      <ExternalIcon size={12} color={colors.muted} />
    </a>
  );
}

function StepTitle({ step, size }: { step: UsageStepRow; size: Size }) {
  return (
    <View style={styles.title}>
      <Text style={styles.number}>{step.number}</Text>
      {step.href === null ? (
        <Text style={[size === "body" ? text.body : text.small, styles.name]}>
          {step.name}
        </Text>
      ) : (
        <StepLink step={step} href={step.href} size={size} />
      )}
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
      <Table
        columns={killed ? [...columns, killColumn] : columns}
        rows={[...steps]}
        rowKey={(step) => String(step.number)}
        cell={cell}
        accessibilityLabel="Steps"
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
  name: { flexShrink: 1, color: colors.ink },
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
