// The Overflow panel of Machines: a row for each connected provider, with its
// account and the spend of the month against its own limit, the confirmed and
// the unconfirmed costs apart, and the overflow wait under the rows. With no
// connected provider, the panel says that jobs run on the machines only.
// Every role sees it; only an Admin gets the link to the overflow settings
// and Connect a provider. A failed read shows in the panel with Try again,
// and the machines stay on the page. A failed refresh keeps the last
// summary, with a warning and Try again above it.

import { Link } from "expo-router";
import { StyleSheet, Text, View } from "react-native";
import type { IconComponent } from "@firna/ui";

import { colors, radii } from "@ciproxy/tokens";

import type { OverflowSettings } from "../../api/machine-schema.ts";
import { ActionButton, ActionLink } from "../../components/actions.tsx";
import { Panel, TextLink } from "../../components/feature-parts.tsx";
import { CloudIcon, CloudOffIcon } from "../../components/icons.tsx";
import { Notice } from "../../components/notice.tsx";
import { WaitingLine } from "../../components/page-state.tsx";
import { text } from "../../components/text.ts";
import type { OverflowSnapshot } from "../../state/machines-overflow.ts";
import { listCopy, overflowCopy } from "../../view/machines-copy.ts";
import {
  overflowModel,
  type ProviderRow,
} from "../../view/machines-overflow.ts";
import { overflowSettingsHref } from "../../view/machines-routes.ts";
import { SpendFiguresView } from "../overflow/spend-figures.tsx";

function Tile({ icon: Icon }: { icon: IconComponent }) {
  return (
    <View style={styles.tile}>
      <Icon size={16} color={colors.ink2} />
    </View>
  );
}

function RefreshFailure({ onRetry }: { onRetry: () => void }) {
  return (
    <Notice
      tone="warning"
      title={overflowCopy.refreshTitle}
      actions={
        <ActionButton label={listCopy.tryAgain} size="sm" onPress={onRetry} />
      }
    >
      {overflowCopy.refreshBody}
    </Notice>
  );
}

function Row({ row, compact }: { row: ProviderRow; compact: boolean }) {
  return (
    <View style={[styles.row, compact && styles.rowCompact]}>
      <Tile icon={CloudIcon} />
      <View style={styles.text}>
        <Text style={[text.body, text.strong]}>{row.name}</Text>
        <Text style={text.caption}>{row.account}</Text>
        {row.hold ? <Text style={text.caption}>{row.hold}</Text> : null}
        {row.note ? <Text style={text.caption}>{row.note}</Text> : null}
      </View>
      {row.spend ? (
        <View style={[styles.meter, compact && styles.meterCompact]}>
          <SpendFiguresView figures={row.spend} compact />
        </View>
      ) : null}
    </View>
  );
}

function NoProvider({
  workspaceId,
  admin,
  compact,
}: {
  workspaceId: string;
  admin: boolean;
  compact: boolean;
}) {
  return (
    <View style={[styles.row, compact && styles.rowCompact]}>
      <Tile icon={CloudOffIcon} />
      <View style={styles.text}>
        <Text style={[text.body, text.strong]}>
          {overflowCopy.machinesOnly}
        </Text>
        <Text style={text.caption}>{overflowCopy.connectLead}</Text>
      </View>
      {admin ? (
        <Link href={overflowSettingsHref(workspaceId)} asChild>
          <ActionLink
            label={overflowCopy.connect}
            href={overflowSettingsHref(workspaceId)}
            size="sm"
          />
        </Link>
      ) : null}
    </View>
  );
}

function Summary({
  summary,
  workspaceId,
  admin,
  compact,
}: {
  summary: OverflowSettings;
  workspaceId: string;
  admin: boolean;
  compact: boolean;
}) {
  const model = overflowModel(summary);
  if (model.kind === "none") {
    return (
      <NoProvider workspaceId={workspaceId} admin={admin} compact={compact} />
    );
  }
  return (
    <View style={styles.rows}>
      {model.rows.map((row, index) => (
        <View key={row.provider} style={index > 0 ? styles.divided : undefined}>
          <Row row={row} compact={compact} />
        </View>
      ))}
      <Text style={text.caption}>{model.caption}</Text>
    </View>
  );
}

export function OverflowPanel({
  snapshot,
  workspaceId,
  admin,
  compact,
  onRetry,
}: {
  snapshot: OverflowSnapshot;
  workspaceId: string;
  admin: boolean;
  compact: boolean;
  onRetry: () => void;
}) {
  let body;
  if (snapshot.status === "loading") {
    body = <WaitingLine>{overflowCopy.loading}</WaitingLine>;
  } else if (snapshot.status === "failed") {
    body = (
      <Notice
        tone="danger"
        title={overflowCopy.failed}
        actions={
          <ActionButton label={listCopy.tryAgain} size="sm" onPress={onRetry} />
        }
      />
    );
  } else {
    body = (
      <Summary
        summary={snapshot.page}
        workspaceId={workspaceId}
        admin={admin}
        compact={compact}
      />
    );
  }
  return (
    <Panel
      title={overflowCopy.title}
      action={
        admin ? (
          <TextLink
            href={overflowSettingsHref(workspaceId)}
            label={overflowCopy.settings}
          />
        ) : undefined
      }
    >
      {snapshot.status === "ready" && snapshot.refreshFailure !== null ? (
        <RefreshFailure onRetry={onRetry} />
      ) : null}
      {body}
    </Panel>
  );
}

const styles = StyleSheet.create({
  rows: { gap: 12 },
  divided: {
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: colors.border,
  },
  row: { flexDirection: "row", alignItems: "center", gap: 12 },
  rowCompact: { flexWrap: "wrap" },
  tile: {
    width: 32,
    height: 32,
    alignItems: "center",
    justifyContent: "center",
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: radii.lg,
    backgroundColor: colors.surface,
  },
  text: { flex: 1, minWidth: 0, gap: 2 },
  meter: { width: 260 },
  meterCompact: { width: "100%" },
});
