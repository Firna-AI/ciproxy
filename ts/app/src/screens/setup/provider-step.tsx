// The open provider step of Setup: a choice of the providers of ciproxy, the
// form of the picked one with its key and the monthly spend limit of that
// provider, and Skip for now. A provider that the deployment cannot start
// shows as unavailable right now. After a provider rejected the saved key,
// the step asks for a new key of that provider only, and the limit starts
// with its saved limit. A deployment that can start no provider, or a failed
// read, leaves Skip for now. A connection that the API accepted is never
// sent again: a failed read after it offers only Try again, which reads.

import { useEffect } from "react";
import { StyleSheet, View } from "react-native";

import type { OverflowSettings, ProviderSettings } from "../../api/schema.ts";
import { ActionButton } from "../../components/actions.tsx";
import { Notice } from "../../components/notice.tsx";
import { WaitingLine } from "../../components/page-state.tsx";
import { useServices } from "../../services/context.tsx";
import { useValue } from "../../services/hooks.ts";
import type { OverflowStore } from "../../state/overflow.ts";
import { rejectedNotice } from "../../view/overflow.ts";
import { providerName } from "../../view/providers.ts";
import {
  ProviderConnect,
  ProviderConnection,
} from "../overflow/provider-choice.tsx";

/** The available provider whose saved key was rejected, if any. */
function rejectedProvider(settings: OverflowSettings): ProviderSettings | null {
  return (
    settings.providers.find(
      (item) => item.available && (item.account?.rejected_at ?? null) !== null,
    ) ?? null
  );
}

function ProviderForm({
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
  const locked = useValue(store.pending);
  const skipAction = (
    <ActionButton label="Skip for now" tone="ghost" onPress={skip} />
  );
  const rejected = rejectedProvider(settings);
  const rejectedAt = rejected?.account?.rejected_at ?? null;
  if (rejected !== null && rejectedAt !== null) {
    const notice = rejectedNotice(rejected, rejectedAt);
    return (
      <View style={styles.body}>
        <Notice tone={notice.tone} title={notice.title}>
          {notice.body}
        </Notice>
        <ProviderConnection
          store={store}
          provider={rejected}
          reconnect
          onConnected={onConnected}
          actions={skipAction}
        />
      </View>
    );
  }
  return (
    <ProviderConnect
      store={store}
      providers={settings.providers}
      locked={locked}
      onConnected={onConnected}
      actions={skipAction}
    />
  );
}

export function ProviderStep({
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
    services.providerSkip(workspaceId).skip(login);
    onResolved();
  };
  const skipAction = (
    <ActionButton label="Skip for now" tone="ghost" onPress={skip} />
  );
  if (snapshot.status === "loading") {
    return <WaitingLine>Loading the overflow providers…</WaitingLine>;
  }
  if (snapshot.status === "failed") {
    const accepted = snapshot.accepted;
    const connected = accepted?.kind === "connected" ? accepted.provider : null;
    return (
      <View style={styles.body}>
        <Notice tone="danger" title="Unable to load the overflow providers">
          {connected === null
            ? "Check the connection, then try again."
            : `${providerName(connected)} connected. Check the connection, then try again.`}
        </Notice>
        <View style={styles.row}>
          <ActionButton
            label="Try again"
            tone="primary"
            onPress={() => void store.load()}
          />
          {connected === null ? skipAction : null}
        </View>
      </View>
    );
  }
  if (!snapshot.settings.providers.some((item) => item.available)) {
    return (
      <View style={styles.body}>
        <Notice tone="info" title="No overflow provider is available right now">
          Connect one later in Settings under Overflow. Jobs run on your
          machines.
        </Notice>
        <View style={styles.row}>{skipAction}</View>
      </View>
    );
  }
  return (
    <ProviderForm
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
