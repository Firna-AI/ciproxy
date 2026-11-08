// The members of the Members tab: each with the role picker, the day of the
// role, and Remove, which the row of the signed-in person does not offer. A
// table on a wide window and cards on a phone. The role picker and the name
// also serve the requests for access.

import { StyleSheet, Text, View } from "react-native";
import { Avatar } from "@firna/ui/avatar";
import { Badge } from "@firna/ui/badge";
import { DropdownSelector } from "@firna/ui/dropdown";
import { Table, type TableColumn } from "@firna/ui/table";

import { colors } from "@ciproxy/tokens";

import type { WorkspaceRole } from "../../api/schema.ts";
import { ActionButton } from "../../components/actions.tsx";
import { text } from "../../components/text.ts";
import { initial } from "../../view/format.ts";
import { roleOptions, type MemberRow } from "../../view/settings-members.ts";

function isRole(value: string): value is WorkspaceRole {
  return roleOptions.some((option) => option.value === value);
}

export function RolePicker({
  name,
  role,
  disabled,
  onPick,
}: {
  name: string;
  role: WorkspaceRole;
  disabled: boolean;
  onPick: (role: WorkspaceRole) => void;
}) {
  return (
    <View style={styles.picker}>
      <DropdownSelector
        variant="field"
        size="sm"
        options={roleOptions}
        value={role}
        readOnly={disabled}
        onValueChange={(value) => {
          if (isRole(value)) onPick(value);
        }}
        triggerLabel={`Role of ${name}`}
      />
    </View>
  );
}

export function PersonName({ name, you }: { name: string; you: boolean }) {
  return (
    <View style={styles.person}>
      <Avatar
        label={initial(name)}
        accessibilityLabel={name}
        size={28}
        tone="soft"
      />
      <Text style={[text.small, text.strong, styles.name]}>{name}</Text>
      {you ? (
        <Badge tone="neutral" variant="outline" size="sm">
          You
        </Badge>
      ) : null}
    </View>
  );
}

export type MemberHandlers = {
  /** The member whose change runs now, and the role that it saves. */
  pending: { id: string; role: WorkspaceRole | null } | null;
  onPick: (row: MemberRow, role: WorkspaceRole) => void;
  onRemove: (row: MemberRow) => void;
};

const columns: TableColumn[] = [
  { key: "member", label: "Member", flex: 1 },
  { key: "role", label: "Role", width: 150 },
  { key: "since", label: "Since", width: 124 },
  { key: "remove", label: "", width: 110, align: "right" },
];

function Remove({
  row,
  handlers,
}: {
  row: MemberRow;
  handlers: MemberHandlers;
}) {
  if (row.you) return null;
  return (
    <ActionButton
      label="Remove"
      accessibilityLabel={`Remove ${row.name}`}
      tone="danger"
      size="sm"
      busy={handlers.pending?.id === row.id && handlers.pending.role === null}
      disabled={handlers.pending !== null}
      onPress={() => handlers.onRemove(row)}
    />
  );
}

function Picker({
  row,
  handlers,
}: {
  row: MemberRow;
  handlers: MemberHandlers;
}) {
  const pending =
    handlers.pending?.id === row.id ? handlers.pending.role : null;
  return (
    <RolePicker
      name={row.name}
      role={pending ?? row.role}
      disabled={handlers.pending !== null}
      onPick={(role) => handlers.onPick(row, role)}
    />
  );
}

export function MemberList({
  rows,
  phone,
  handlers,
  loading = false,
}: {
  rows: readonly MemberRow[];
  phone: boolean;
  handlers: MemberHandlers;
  /** Skeleton rows in the layout of the table, while the members load. */
  loading?: boolean;
}) {
  if (!phone || loading) {
    return (
      <Table
        accessibilityLabel="Members"
        columns={columns}
        rows={[...rows]}
        loading={loading}
        loadingRowCount={3}
        rowKey={(row) => row.id}
        cell={(row, key) => {
          if (key === "member")
            return <PersonName name={row.name} you={row.you} />;
          if (key === "role") return <Picker row={row} handlers={handlers} />;
          if (key === "since")
            return <Text style={text.caption}>{row.since}</Text>;
          return <Remove row={row} handlers={handlers} />;
        }}
      />
    );
  }
  return (
    <View role="list" aria-label="Members">
      {rows.map((row, index) => (
        <View
          key={row.id}
          role="listitem"
          style={[styles.card, index === 0 && styles.first]}
        >
          <View style={styles.spread}>
            <PersonName name={row.name} you={row.you} />
            <Picker row={row} handlers={handlers} />
          </View>
          <View style={styles.spread}>
            <Text style={text.caption}>Since {row.since}</Text>
            <Remove row={row} handlers={handlers} />
          </View>
        </View>
      ))}
    </View>
  );
}

const styles = StyleSheet.create({
  picker: { minWidth: 120 },
  person: { flexDirection: "row", alignItems: "center", gap: 10, minWidth: 0 },
  name: { flexShrink: 1 },
  card: {
    gap: 10,
    paddingVertical: 12,
    paddingHorizontal: 14,
    borderTopWidth: 1,
    borderTopColor: colors.border,
  },
  first: { borderTopWidth: 0 },
  spread: {
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "center",
    justifyContent: "space-between",
    gap: 10,
  },
});
