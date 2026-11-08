// The labels of the workspace: a table on a wide window and cards on a
// phone, each with its name, its size under the name, where its jobs can
// run, and Edit. The Boat part is the switch of the label, because the API
// gives no Boat fit. The default label carries a tag, because it is the one
// label that stays.

import { StyleSheet, Text, View } from "react-native";
import { Badge } from "@firna/ui/badge";
import { Table, type TableColumn } from "@firna/ui/table";

import { colors } from "@ciproxy/tokens";

import { ActionButton } from "../../components/actions.tsx";
import { StatusLabel } from "../../components/feature-parts.tsx";
import { text } from "../../components/text.ts";
import type { LabelRow } from "../../view/settings-labels.ts";

const columns: TableColumn[] = [
  { key: "label", label: "Label", flex: 3 },
  { key: "where", label: "Where jobs run", flex: 2 },
  { key: "edit", label: "", width: 96, align: "right" },
];

function LabelName({ row }: { row: LabelRow }) {
  return (
    <View style={styles.nameRow}>
      <Text style={[text.small, text.mono, text.strong]}>{row.name}</Text>
      {row.isDefault ? (
        <Badge tone="neutral" variant="outline" size="sm">
          Default
        </Badge>
      ) : null}
    </View>
  );
}

function Where({ row }: { row: LabelRow }) {
  return (
    <View style={styles.where}>
      {row.where.warn ? (
        <StatusLabel tone="warn">{row.where.text}</StatusLabel>
      ) : (
        <Text style={text.small}>{row.where.text}</Text>
      )}
      {row.where.boat ? (
        <Text style={text.caption}>{row.where.boat}</Text>
      ) : null}
    </View>
  );
}

function Edit({
  row,
  onEdit,
}: {
  row: LabelRow;
  onEdit: (id: string) => void;
}) {
  return (
    <ActionButton
      label="Edit"
      accessibilityLabel={`Edit ${row.name}`}
      size="sm"
      onPress={() => onEdit(row.id)}
    />
  );
}

export function LabelList({
  rows,
  phone,
  loading = false,
  onEdit,
}: {
  rows: readonly LabelRow[];
  phone: boolean;
  loading?: boolean;
  onEdit: (id: string) => void;
}) {
  if (!phone || loading) {
    return (
      <Table
        accessibilityLabel="Labels"
        columns={columns}
        rows={[...rows]}
        rowKey={(row) => row.id}
        loading={loading}
        loadingRowCount={3}
        cell={(row, key) => {
          if (key === "label") {
            return (
              <View style={styles.cell}>
                <LabelName row={row} />
                <Text style={text.caption}>{row.size}</Text>
              </View>
            );
          }
          if (key === "where") return <Where row={row} />;
          return <Edit row={row} onEdit={onEdit} />;
        }}
      />
    );
  }
  return (
    <View role="list" aria-label="Labels">
      {rows.map((row, index) => (
        <View
          key={row.id}
          role="listitem"
          style={[styles.card, index === 0 && styles.cardFirst]}
        >
          <View style={styles.cardText}>
            <LabelName row={row} />
            <Text style={text.small}>{row.size}</Text>
            <Where row={row} />
          </View>
          <Edit row={row} onEdit={onEdit} />
        </View>
      ))}
    </View>
  );
}

const styles = StyleSheet.create({
  cell: { gap: 2, paddingVertical: 6, minWidth: 0 },
  where: { gap: 2, paddingVertical: 6, minWidth: 0 },
  nameRow: {
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "center",
    gap: 8,
  },
  card: {
    flexDirection: "row",
    alignItems: "flex-start",
    justifyContent: "space-between",
    gap: 12,
    paddingVertical: 12,
    paddingHorizontal: 14,
    borderTopWidth: 1,
    borderTopColor: colors.border,
  },
  cardFirst: { borderTopWidth: 0 },
  cardText: { flexShrink: 1, gap: 4, minWidth: 0 },
});
