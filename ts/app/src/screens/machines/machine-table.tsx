// The own machines as a table on a wide or narrow window and as a dense list
// on a phone. Each machine name is a link to its page, in the mono face,
// with its threads and memory beside it. A long name wraps inside its column,
// and on a phone the health stays beside it. An offline machine shows no
// slots.

import { StyleSheet, Text, View } from "react-native";

import { colors } from "@ciproxy/tokens";

import type { MachineView } from "../../api/schema.ts";
import { text } from "../../components/text.ts";
import type { Viewport } from "../../components/viewport.ts";
import { listCopy, statusWords } from "../../view/machines-copy.ts";
import { machineHref } from "../../view/machines-routes.ts";
import {
  cacheText,
  hardwareText,
  lastSeenText,
  machineStatus,
  slotCaption,
  slotStates,
} from "../../view/machines.ts";
import { SlotLegend, SlotStrip } from "./machine-figures.tsx";
import { StatusLabel, TextLink } from "../../components/feature-parts.tsx";
import { statusTones } from "./machine-status.ts";

type Props = {
  machines: readonly MachineView[];
  workspaceId: string;
  now: number;
};

function Name({
  machine,
  workspaceId,
  stacked,
}: {
  machine: MachineView;
  workspaceId: string;
  stacked: boolean;
}) {
  return (
    <View style={[styles.name, stacked ? styles.nameStacked : styles.nameRow]}>
      <TextLink
        href={machineHref(workspaceId, machine.id)}
        label={machine.hostname}
        mono
      />
      <Text style={text.caption}>{hardwareText(machine)}</Text>
    </View>
  );
}

function Status({ machine }: { machine: MachineView }) {
  const status = machineStatus(machine);
  return (
    <StatusLabel tone={statusTones[status]}>{statusWords[status]}</StatusLabel>
  );
}

function Slots({
  machine,
  stacked,
}: {
  machine: MachineView;
  stacked: boolean;
}) {
  const caption = slotCaption(machine);
  const states = machine.offline ? [] : slotStates(machine);
  return (
    <View style={[styles.slots, stacked && styles.slotsStacked]}>
      {states.length > 0 ? <SlotStrip states={states} label={caption} /> : null}
      <Text style={text.caption}>{caption}</Text>
    </View>
  );
}

type Column = {
  key: string;
  label: string;
  width?: number;
  flex?: number;
  right?: boolean;
};

const columns: readonly Column[] = [
  { key: "machine", label: listCopy.columns.machine, flex: 2 },
  { key: "health", label: listCopy.columns.health, width: 112 },
  { key: "slots", label: listCopy.columns.slots, flex: 2 },
  { key: "cache", label: listCopy.columns.cache, width: 124 },
  { key: "seen", label: listCopy.columns.seen, width: 96, right: true },
];

function cellStyle(column: Column) {
  return [
    styles.cell,
    column.flex ? { flex: column.flex, minWidth: 0 } : { width: column.width },
    column.right && styles.right,
  ];
}

function Cell({
  machine,
  column,
  props,
  stacked,
}: {
  machine: MachineView;
  column: Column;
  props: Props;
  stacked: boolean;
}) {
  switch (column.key) {
    case "machine":
      return (
        <Name
          machine={machine}
          workspaceId={props.workspaceId}
          stacked={stacked}
        />
      );
    case "health":
      return <Status machine={machine} />;
    case "slots":
      return <Slots machine={machine} stacked={stacked} />;
    case "cache":
      return <Text style={[text.small, styles.ink]}>{cacheText(machine)}</Text>;
    default:
      return <Text style={text.small}>{lastSeenText(machine, props.now)}</Text>;
  }
}

/** The table of a wide or narrow window; a narrow one stacks each cell. */
export function MachineTable(props: Props & { viewport: Viewport }) {
  const stacked = props.viewport !== "wide";
  if (props.viewport === "mobile") return <MachineCards {...props} />;
  return (
    <View role="table" aria-label={listCopy.ownMachines}>
      <View role="row" style={[styles.tr, styles.head]}>
        {columns.map((column) => (
          <View key={column.key} role="columnheader" style={cellStyle(column)}>
            <Text style={styles.th}>{column.label}</Text>
          </View>
        ))}
      </View>
      {props.machines.map((machine) => (
        <View key={machine.id} role="row" style={[styles.tr, styles.bodyRow]}>
          {columns.map((column) => (
            <View key={column.key} role="cell" style={cellStyle(column)}>
              <Cell
                machine={machine}
                column={column}
                props={props}
                stacked={stacked}
              />
            </View>
          ))}
        </View>
      ))}
      <View style={styles.legend}>
        <SlotLegend />
      </View>
    </View>
  );
}

function MachineCards(props: Props) {
  return (
    <View role="list" aria-label={listCopy.ownMachines}>
      {props.machines.map((machine, index) => (
        <View
          key={machine.id}
          role="listitem"
          style={[styles.card, index > 0 && styles.cardDivider]}
        >
          <View style={styles.cardHead}>
            <View style={styles.cardName}>
              <Name machine={machine} workspaceId={props.workspaceId} stacked />
            </View>
            <Status machine={machine} />
          </View>
          <Slots machine={machine} stacked={false} />
        </View>
      ))}
      <View style={[styles.legend, styles.legendCard]}>
        <SlotLegend />
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  tr: { flexDirection: "row", alignItems: "center" },
  head: {
    minHeight: 36,
    borderBottomWidth: 1,
    borderBottomColor: colors.border,
    backgroundColor: colors.bg,
  },
  bodyRow: {
    minHeight: 44,
    borderBottomWidth: 1,
    borderBottomColor: colors.border,
  },
  cell: { paddingHorizontal: 16, paddingVertical: 8 },
  right: { alignItems: "flex-end" },
  th: { fontSize: 12, fontWeight: "500", color: colors.muted },
  ink: { color: colors.ink },
  name: { minWidth: 0 },
  nameRow: {
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "baseline",
    columnGap: 10,
  },
  nameStacked: { gap: 2 },
  slots: {
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "center",
    gap: 12,
  },
  slotsStacked: { flexDirection: "column", alignItems: "flex-start", gap: 6 },
  legend: { paddingVertical: 10, paddingHorizontal: 16 },
  legendCard: {
    borderTopWidth: 1,
    borderTopColor: colors.border,
    paddingHorizontal: 14,
  },
  card: { gap: 8, paddingVertical: 12, paddingHorizontal: 14 },
  cardDivider: { borderTopWidth: 1, borderTopColor: colors.border },
  cardHead: {
    flexDirection: "row",
    alignItems: "flex-start",
    justifyContent: "space-between",
    gap: 8,
  },
  cardName: { flexGrow: 1, flexShrink: 1, flexBasis: 0, minWidth: 0 },
});
