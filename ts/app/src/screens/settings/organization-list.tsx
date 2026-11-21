// The GitHub organizations of the workspace: a table on a wide window and
// cards on a phone. A linked row shows the day it joined, its repository
// access, Repositories, and Take out while another linked organization stays. An inactive row says
// why under its name, with the action that clears its hold. A removed row
// shows the deadline of a new install that keeps its job history, and
// Install again.

import { Link } from "expo-router";
import { StyleSheet, Text, View } from "react-native";
import { Avatar } from "@firna/ui/avatar";
import { Table, type TableColumn } from "@firna/ui/table";

import { colors } from "@ciproxy/tokens";

import { ActionButton, ActionLink } from "../../components/actions.tsx";
import { StatusLabel } from "../../components/feature-parts.tsx";
import { text } from "../../components/text.ts";
import { initial } from "../../view/format.ts";
import type { OrganizationRow } from "../../view/settings-organizations.ts";
import { repositoriesHref } from "../../view/settings-tabs.ts";

export type RowHandlers = {
  workspaceId: string;
  onTakeOut: (row: OrganizationRow) => void;
  onTurnOn: (row: OrganizationRow) => void;
  onInstallAgain: () => void;
  /** Install again runs now. */
  installing: boolean;
};

const columns: TableColumn[] = [
  { key: "organization", label: "Organization", flex: 2 },
  { key: "access", label: "Repository access", flex: 2 },
  { key: "actions", label: "", width: 240, align: "right" },
];

function OrganizationName({ row }: { row: OrganizationRow }) {
  return (
    <View style={styles.person}>
      <Avatar
        label={initial(row.login)}
        accessibilityLabel={row.login}
        size={28}
        shape="square"
      />
      <View style={styles.personText}>
        <Text style={[text.small, text.strong]}>{row.login}</Text>
        {row.since === null ? null : (
          <Text style={text.caption}>{row.since}</Text>
        )}
        {row.status ? (
          <View style={styles.status}>
            <StatusLabel tone={row.status.tone}>{row.status.text}</StatusLabel>
            {row.status.lines.map((line) => (
              <Text key={line} style={text.caption}>
                {line}
              </Text>
            ))}
          </View>
        ) : null}
      </View>
    </View>
  );
}

function RowActions({
  row,
  handlers,
  phone,
}: {
  row: OrganizationRow;
  handlers: RowHandlers;
  phone: boolean;
}) {
  const align = phone ? styles.start : styles.end;
  if (row.installAgain) {
    return (
      <View style={[styles.actions, align]}>
        <ActionButton
          label="Install again"
          accessibilityLabel={`Install the App again on ${row.login}`}
          size="sm"
          busy={handlers.installing}
          onPress={handlers.onInstallAgain}
        />
      </View>
    );
  }
  const repositories = repositoriesHref(handlers.workspaceId, row.id);
  return (
    <View style={[styles.stack, phone ? styles.stackStart : styles.stackEnd]}>
      {row.chooseTeam || row.turnOn ? (
        <View style={[styles.actions, align]}>
          {row.chooseTeam ? (
            <Link href={row.chooseTeam} asChild>
              <ActionLink label="Choose Team" href={row.chooseTeam} size="sm" />
            </Link>
          ) : (
            <ActionButton
              label="Turn on"
              accessibilityLabel={`Turn on ${row.login}`}
              size="sm"
              onPress={() => handlers.onTurnOn(row)}
            />
          )}
        </View>
      ) : null}
      <View style={[styles.actions, align]}>
        {row.repositories ? (
          // The repositories stay on /settings and change only the query,
          // so the link pushes a new history entry for Back.
          <Link href={repositories} asChild push>
            <ActionLink
              label="Repositories"
              accessibilityLabel={`Repositories of ${row.login}`}
              href={repositories}
              size="sm"
            />
          </Link>
        ) : null}
        {row.takeOut ? (
          <ActionButton
            label="Take out"
            accessibilityLabel={`Take out ${row.login}`}
            tone="danger"
            size="sm"
            onPress={() => handlers.onTakeOut(row)}
          />
        ) : null}
      </View>
    </View>
  );
}

/** Skeleton rows in the layout of the table, while the settings load. */
export function OrganizationListLoading() {
  return (
    <Table
      accessibilityLabel="Organizations"
      columns={columns}
      rows={[]}
      rowKey={() => ""}
      loading
      loadingRowCount={2}
      cell={() => null}
    />
  );
}

export function OrganizationList({
  rows,
  phone,
  handlers,
}: {
  rows: readonly OrganizationRow[];
  phone: boolean;
  handlers: RowHandlers;
}) {
  if (!phone) {
    return (
      <Table
        accessibilityLabel="Organizations"
        columns={columns}
        rows={[...rows]}
        rowKey={(row) => row.id}
        cell={(row, key) => {
          if (key === "organization") return <OrganizationName row={row} />;
          if (key === "access")
            return <Text style={text.small}>{row.access}</Text>;
          return <RowActions row={row} handlers={handlers} phone={false} />;
        }}
      />
    );
  }
  return (
    <View role="list" aria-label="Organizations">
      {rows.map((row, index) => (
        <View
          key={row.id}
          role="listitem"
          style={[styles.card, index === 0 && styles.cardFirst]}
        >
          <OrganizationName row={row} />
          <Text style={text.small}>{row.access}</Text>
          <RowActions row={row} handlers={handlers} phone />
        </View>
      ))}
    </View>
  );
}

const styles = StyleSheet.create({
  person: {
    flexDirection: "row",
    alignItems: "flex-start",
    gap: 10,
    paddingVertical: 8,
    minWidth: 0,
  },
  personText: { flexShrink: 1, gap: 2, minWidth: 0 },
  status: { gap: 2, marginTop: 2 },
  stack: { gap: 8, paddingVertical: 8 },
  stackEnd: { alignItems: "flex-end" },
  stackStart: { alignItems: "flex-start" },
  actions: { flexDirection: "row", flexWrap: "wrap", gap: 8 },
  end: { justifyContent: "flex-end" },
  start: { justifyContent: "flex-start" },
  card: {
    gap: 8,
    paddingVertical: 12,
    paddingHorizontal: 14,
    borderTopWidth: 1,
    borderTopColor: colors.border,
  },
  cardFirst: { borderTopWidth: 0 },
});
