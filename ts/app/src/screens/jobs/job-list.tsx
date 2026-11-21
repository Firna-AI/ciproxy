// The job lists: a table on a wide window and a dense list on a phone. A live
// row leads with its status dot. A running job shows where it runs, and a
// waiting job shows its label, because no machine holds it yet. A finished
// job shows its result, where it ran, its wait, its duration, and when it
// finished, and its name opens its page.

import type { ReactNode } from "react";
import { StyleSheet, Text, View } from "react-native";
import { SkeletonBar, SkeletonGroup } from "@firna/ui/skeleton";
import { Table, type TableColumn } from "@firna/ui/table";

import { colors, fonts, radii } from "@ciproxy/tokens";

import { StatusLabel, StatusMark } from "../../components/status-label.tsx";
import { text } from "../../components/text.ts";
import { TextLink } from "../../components/text-link.tsx";
import { useViewport } from "../../components/viewport.ts";
import type { FinishedRow } from "../../view/jobs-history.ts";
import type { LiveRow } from "../../view/jobs-live.ts";
import { PlaceLabel } from "./place-label.tsx";

export type LiveKind = "running" | "waiting";

const liveColumns: Readonly<Record<LiveKind, TableColumn[]>> = {
  running: [
    { key: "job", label: "Job", flex: 3 },
    { key: "place", label: "Where", flex: 2 },
    { key: "time", label: "Running for", width: 110, align: "right" },
  ],
  waiting: [
    { key: "job", label: "Job", flex: 3 },
    { key: "label", label: "Label", flex: 1 },
    { key: "time", label: "Waiting for", width: 110, align: "right" },
  ],
};

const finishedColumns: TableColumn[] = [
  { key: "result", label: "Result", width: 120 },
  { key: "job", label: "Job", flex: 3 },
  { key: "place", label: "Where", flex: 2 },
  { key: "waited", label: "Waited", width: 80, align: "right" },
  { key: "duration", label: "Duration", width: 92, align: "right" },
  { key: "finished", label: "Finished", width: 84, align: "right" },
];

function JobName({
  name,
  context,
  link,
}: {
  name: string;
  context: string;
  link?: string;
}) {
  return (
    <View style={styles.name}>
      {link ? (
        <TextLink label={name} href={link} strong size="small" />
      ) : (
        <Text style={[text.small, text.strong]}>{name}</Text>
      )}
      <Text style={text.caption} numberOfLines={2}>
        {context}
      </Text>
    </View>
  );
}

function Mono({ children }: { children: string | null }) {
  return <Text style={[text.small, styles.mono]}>{children ?? ""}</Text>;
}

function LabelName({ name }: { name: string }) {
  return (
    <Text style={[text.caption, styles.label]} numberOfLines={1}>
      {name}
    </Text>
  );
}

function liveCell(row: LiveRow, key: string, kind: LiveKind): ReactNode {
  switch (key) {
    case "job":
      return (
        <View style={styles.lead}>
          <StatusMark
            tone={kind === "running" ? "run" : "warn"}
            label={kind === "running" ? "Running" : "Waiting"}
          />
          <JobName name={row.name} context={row.context} />
        </View>
      );
    case "place":
      return <PlaceLabel place={row.place} slot />;
    case "label":
      return <LabelName name={row.label} />;
    default:
      return <Mono>{row.time}</Mono>;
  }
}

export function LiveList({
  kind,
  rows,
}: {
  kind: LiveKind;
  rows: readonly LiveRow[];
}) {
  const mobile = useViewport() === "mobile";
  if (!mobile) {
    return (
      <Table
        columns={liveColumns[kind]}
        rows={[...rows]}
        rowKey={(row) => row.id}
        cell={(row, key) => liveCell(row, key, kind)}
        accessibilityLabel={
          kind === "running" ? "Running jobs" : "Waiting jobs"
        }
      />
    );
  }
  return (
    <View>
      {rows.map((row, index) => (
        <View
          key={row.id}
          style={[styles.mobileRow, index > 0 && styles.divided]}
        >
          {liveCell(row, "job", kind)}
          <View style={styles.meta}>
            {kind === "running" ? (
              <PlaceLabel place={row.place} slot />
            ) : (
              <LabelName name={row.label} />
            )}
            <Mono>{row.time}</Mono>
          </View>
        </View>
      ))}
    </View>
  );
}

function finishedCell(row: FinishedRow, key: string): ReactNode {
  switch (key) {
    case "result":
      return <StatusLabel tone={row.look.tone}>{row.look.label}</StatusLabel>;
    case "job":
      return <JobName name={row.name} context={row.context} link={row.href} />;
    case "place":
      return <PlaceLabel place={row.place} slot={false} />;
    case "waited":
      return <Mono>{row.waited}</Mono>;
    case "duration":
      return <Mono>{row.duration}</Mono>;
    default:
      return <Text style={[text.small, styles.muted]}>{row.finished}</Text>;
  }
}

export function FinishedList({ rows }: { rows: readonly FinishedRow[] }) {
  const mobile = useViewport() === "mobile";
  if (!mobile) {
    return (
      <Table
        columns={finishedColumns}
        rows={[...rows]}
        rowKey={(row) => row.id}
        cell={finishedCell}
        accessibilityLabel="Finished jobs"
      />
    );
  }
  return (
    <View>
      {rows.map((row, index) => (
        <View
          key={row.id}
          style={[styles.mobileRow, index > 0 && styles.divided]}
        >
          <View style={styles.meta}>
            <JobName name={row.name} context={row.context} link={row.href} />
            <Mono>{row.duration}</Mono>
          </View>
          <View style={styles.facts}>
            {finishedCell(row, "result")}
            <PlaceLabel place={row.place} slot={false} />
            <Text style={text.caption}>waited {row.waited}</Text>
          </View>
        </View>
      ))}
    </View>
  );
}

/** A list that is still loading: skeleton rows in the place of the jobs. */
export function LoadingList({ kind }: { kind: LiveKind | "finished" }) {
  const mobile = useViewport() === "mobile";
  if (!mobile) {
    return (
      <Table
        columns={kind === "finished" ? finishedColumns : liveColumns[kind]}
        rows={[]}
        rowKey={(_row, index) => String(index)}
        cell={() => null}
        loading
        loadingRowCount={6}
      />
    );
  }
  return (
    <View style={styles.mobileLoading}>
      <SkeletonGroup direction="column" gap={14}>
        {[0, 1, 2, 3, 4, 5].map((index) => (
          <SkeletonBar key={index} width="90%" height={14} />
        ))}
      </SkeletonGroup>
    </View>
  );
}

const styles = StyleSheet.create({
  name: { flexShrink: 1, minWidth: 0, gap: 1 },
  lead: { flexDirection: "row", alignItems: "center", gap: 10, minWidth: 0 },
  mono: { fontFamily: fonts.mono, color: colors.ink },
  muted: { color: colors.muted },
  label: {
    alignSelf: "flex-start",
    paddingHorizontal: 6,
    borderRadius: radii.sm,
    backgroundColor: colors.soft,
    color: colors.ink,
    fontFamily: fonts.mono,
  },
  mobileRow: { gap: 6, paddingVertical: 12, paddingHorizontal: 14 },
  divided: { borderTopWidth: 1, borderTopColor: colors.border },
  meta: {
    flexDirection: "row",
    alignItems: "flex-start",
    justifyContent: "space-between",
    gap: 12,
  },
  facts: {
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "center",
    gap: 12,
  },
  mobileLoading: { padding: 14 },
});
