// The open Boat step of Setup: the key and the monthly limit, Connect Boat,
// and Skip for now. After Boat rejected the saved key, the step asks for a
// new key, and the limit starts with the saved limit. A deployment without
// the Boat service, or a failed read, leaves Skip for now. A connection that
// the API accepted is never sent again: a failed read after it offers only
// Try again, which reads.

import { useEffect } from "react";
import { StyleSheet, Text, View } from "react-native";

import { ActionButton } from "../../components/actions.tsx";
import { Notice } from "../../components/notice.tsx";
import { WaitingLine } from "../../components/page-state.tsx";
import { text } from "../../components/text.ts";
import { useServices } from "../../services/context.tsx";
import { useValue } from "../../services/hooks.ts";
import type { OverflowStore } from "../../state/overflow.ts";
import type { OverflowSettings } from "../../api/schema.ts";
import { rejectedNotice } from "../../view/boat.ts";
import { ConnectFormView } from "../boat/connect-form.tsx";
import { useConnectForm } from "../boat/use-connect-form.ts";

const intro =
  "When your machines are busy, ciproxy starts jobs in your own Boat account. Boat bills you by the second.";

function BoatForm({
  store,
  settings,
  skip,
  onConnected,
}: {
  store: OverflowStore;
  settings: OverflowSettings;
  skip: () => void;
  onConnected: () => void;
}) {
  const form = useConnectForm(
    store,
    settings.spend_limit_cents ?? null,
    onConnected,
  );
  const rejectedAt = settings.account?.rejected_at ?? null;
  const notice = rejectedAt === null ? null : rejectedNotice(rejectedAt);
  return (
    <View style={styles.body}>
      {notice ? (
        <Notice tone={notice.tone} title={notice.title}>
          {notice.body}
        </Notice>
      ) : (
        <Text style={text.small}>{intro}</Text>
      )}
      <ConnectFormView
        form={form}
        reconnect={notice !== null}
        actions={
          <ActionButton label="Skip for now" tone="ghost" onPress={skip} />
        }
      />
    </View>
  );
}

export function BoatStep({
  workspaceId,
  login,
  onResolved,
}: {
  workspaceId: string;
  login: string;
  /** Called after a skip or a connection, when the next step opens. */
  onResolved: () => void;
}) {
  const services = useServices();
  const store = services.overflow(workspaceId);
  const snapshot = useValue(store.state);
  useEffect(() => {
    void store.ensure();
  }, [store]);
  const skip = () => {
    services.boatSkip(workspaceId).skip(login);
    onResolved();
  };
  const skipAction = (
    <ActionButton label="Skip for now" tone="ghost" onPress={skip} />
  );
  if (snapshot.status === "loading") {
    return <WaitingLine>Loading Boat…</WaitingLine>;
  }
  if (snapshot.status === "failed") {
    const connected = snapshot.accepted === "connected";
    return (
      <View style={styles.body}>
        <Notice tone="danger" title="Unable to load Boat">
          {connected
            ? "Boat connected. Check the connection, then try again."
            : "Check the connection, then try again."}
        </Notice>
        <View style={styles.row}>
          <ActionButton
            label="Try again"
            tone="primary"
            onPress={() => void store.load()}
          />
          {connected ? null : skipAction}
        </View>
      </View>
    );
  }
  if (!snapshot.settings.available) {
    return (
      <View style={styles.body}>
        <Notice tone="info" title="Boat is unavailable right now">
          Connect Boat later in Settings under Overflow. Jobs run on your
          machines.
        </Notice>
        <View style={styles.row}>{skipAction}</View>
      </View>
    );
  }
  return (
    <BoatForm
      key={workspaceId}
      store={store}
      settings={snapshot.settings}
      skip={skip}
      onConnected={() => {
        if (store.state.get().status === "ready") onResolved();
      }}
    />
  );
}

const styles = StyleSheet.create({
  body: { gap: 12 },
  row: { flexDirection: "row", flexWrap: "wrap", gap: 8 },
});
