// The Overflow panel of Machines: where jobs go when the own machines are
// busy, and the Boat spend of the month against its limit. Every role sees
// it; only an Admin gets the link to the overflow settings. A failed read
// shows in the panel with Try again, and the machines stay on the page.

import { StyleSheet, Text, View } from "react-native";

import { colors, radii } from "@ciproxy/tokens";

import { ActionButton } from "../../components/actions.tsx";
import { Panel, TextLink } from "../../components/feature-parts.tsx";
import { Notice } from "../../components/notice.tsx";
import { WaitingLine } from "../../components/page-state.tsx";
import { text } from "../../components/text.ts";
import type { OverflowSnapshot } from "../../state/machines-overflow.ts";
import { listCopy, overflowCopy } from "../../view/machines-copy.ts";
import { overflowModel } from "../../view/machines-overflow.ts";
import { overflowSettingsHref } from "../../view/machines-routes.ts";
import { Meter } from "./machine-figures.tsx";

function CloudIcon() {
  return (
    <svg
      aria-hidden="true"
      width={16}
      height={16}
      viewBox="0 0 24 24"
      fill="none"
      stroke={colors.ink2}
      strokeWidth={1.75}
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M7 18a4.5 4.5 0 0 1-.6-8.96A6 6 0 0 1 18 9.5a4.25 4.25 0 0 1-.5 8.5z" />
    </svg>
  );
}

function Body({
  snapshot,
  onRetry,
  compact,
}: {
  snapshot: OverflowSnapshot;
  onRetry: () => void;
  compact: boolean;
}) {
  if (snapshot.status === "loading") {
    return <WaitingLine>{overflowCopy.loading}</WaitingLine>;
  }
  if (snapshot.status === "failed") {
    return (
      <Notice
        tone="danger"
        title={overflowCopy.failed}
        actions={
          <ActionButton label={listCopy.tryAgain} size="sm" onPress={onRetry} />
        }
      />
    );
  }
  const model = overflowModel(snapshot.summary);
  return (
    <View style={[styles.row, compact && styles.rowCompact]}>
      <View style={styles.tile}>
        <CloudIcon />
      </View>
      <View style={styles.text}>
        <Text style={[text.body, text.strong]}>{overflowCopy.boat}</Text>
        <Text style={text.caption}>{model.caption}</Text>
        {model.note ? <Text style={text.caption}>{model.note}</Text> : null}
      </View>
      {model.meter ? (
        <View style={[styles.meter, compact && styles.meterCompact]}>
          <Meter
            label={overflowCopy.spent}
            ratio={model.meter.ratio}
            value={model.meter.value}
            detail={model.meter.detail}
          />
        </View>
      ) : null}
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
      <Body snapshot={snapshot} onRetry={onRetry} compact={compact} />
    </Panel>
  );
}

const styles = StyleSheet.create({
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
