// The panel of one provider on Settings under Overflow, from its panel model.
// A connected provider shows the name of its account, never the key, with
// Change key and Disconnect, and its own monthly spend limit with the spend
// of the month. While the deployment has no settings for the provider, the
// panel keeps the account, the limit, and Disconnect, and hides Change key,
// because no new connection can start. After the provider rejected the key,
// the panel asks for a new key, with Disconnect. An available provider that
// the workspace has no account at shows what it is and Connect. Every
// connection asks for the key and the limit, and a reconnect and a change of
// the key start the limit with the saved limit. Every action and Cancel wait
// while any overflow change of the workspace runs.

import { useEffect, useRef, useState, type ReactNode } from "react";
import { StyleSheet, Text, View } from "react-native";

import { colors } from "@ciproxy/tokens";

import type { ProviderSettings } from "../../api/schema.ts";
import { ActionButton } from "../../components/actions.tsx";
import { PlusIcon } from "../../components/icons.tsx";
import { Panel } from "../../components/panel.tsx";
import { text } from "../../components/text.ts";
import { useViewport } from "../../components/viewport.ts";
import { useValue } from "../../services/hooks.ts";
import type { OverflowStore } from "../../state/overflow.ts";
import { providerPanelModel } from "../../view/overflow.ts";
import { providerCopy, providerName } from "../../view/providers.ts";
import { LimitSection } from "./limit-section.tsx";
import { ProviderConnection } from "./provider-choice.tsx";

type Mode = "shown" | "change-key" | "connect";

function Account({
  provider,
  name,
  changeKey,
  locked,
  onChangeKey,
  onDisconnect,
}: {
  provider: ProviderSettings;
  name: string;
  changeKey: boolean;
  locked: boolean;
  onChangeKey: () => void;
  onDisconnect: () => void;
}) {
  const mobile = useViewport() === "mobile";
  const brand = providerName(provider.provider);
  return (
    <View style={[styles.spread, mobile && styles.stacked]}>
      <View style={styles.spreadText}>
        <Text style={[text.body, text.strong]}>
          {`Connected to the account ${name}`}
        </Text>
        <Text style={text.caption}>
          {provider.available
            ? `ciproxy starts jobs in this account. ${brand} bills it.`
            : `${brand} is unavailable right now, so jobs skip it. ciproxy keeps the account and its limit.`}
        </Text>
      </View>
      <View style={styles.row}>
        {changeKey ? (
          <ActionButton
            label="Change key"
            size="sm"
            disabled={locked}
            onPress={onChangeKey}
          />
        ) : null}
        <ActionButton
          label="Disconnect"
          size="sm"
          tone="danger"
          disabled={locked}
          onPress={onDisconnect}
        />
      </View>
    </View>
  );
}

/** An available provider that has no account in the workspace. */
function NotConnected({
  provider,
  locked,
  onConnect,
}: {
  provider: ProviderSettings;
  locked: boolean;
  onConnect: () => void;
}) {
  const mobile = useViewport() === "mobile";
  const copy = providerCopy[provider.provider];
  return (
    <View style={[styles.spread, mobile && styles.stacked]}>
      <View style={styles.spreadText}>
        <Text style={text.small}>{copy.summary}</Text>
        <Text style={text.caption}>
          {provider.offered
            ? `Your labels keep ${copy.name} in their order and skip it until you connect it again.`
            : "When you connect it, it goes last in the order of each Linux label."}
        </Text>
      </View>
      <ActionButton
        label={`Connect ${copy.name}`}
        size="sm"
        icon={PlusIcon}
        disabled={locked}
        onPress={onConnect}
      />
    </View>
  );
}

export function ProviderPanel({
  store,
  provider,
  focusRequest,
  onDisconnect,
}: {
  store: OverflowStore;
  provider: ProviderSettings;
  /** A count that moves focus to the title each time it grows. */
  focusRequest: number;
  onDisconnect: () => void;
}) {
  const title = useRef<View>(null);
  const [mode, setMode] = useState<Mode>("shown");
  useEffect(() => {
    if (focusRequest > 0) title.current?.focus();
  }, [focusRequest]);
  const locked = useValue(store.pending);
  const model = providerPanelModel(provider);
  const name = providerName(provider.provider);
  const done = () => {
    setMode("shown");
    title.current?.focus();
  };
  const cancel = (
    <ActionButton
      label="Cancel"
      tone="ghost"
      disabled={locked}
      onPress={done}
    />
  );
  const limit = provider.spend_limit_cents ?? null;
  let body: ReactNode;
  if (model.kind === "none") {
    body =
      mode === "connect" ? (
        <>
          {provider.offered ? null : (
            <Text style={text.small}>
              It goes last in the order of each Linux label. You can move it or
              remove it in each label.
            </Text>
          )}
          <ProviderConnection
            store={store}
            provider={provider}
            reconnect={false}
            autoFocus
            onConnected={done}
            actions={cancel}
          />
        </>
      ) : (
        <NotConnected
          provider={provider}
          locked={locked}
          onConnect={() => setMode("connect")}
        />
      );
  } else if (model.kind === "rejected" && provider.available) {
    body = (
      <ProviderConnection
        store={store}
        provider={provider}
        reconnect
        onConnected={done}
        actions={
          <ActionButton
            label="Disconnect"
            tone="danger"
            disabled={locked}
            onPress={onDisconnect}
          />
        }
      />
    );
  } else if (mode === "change-key" && model.actions.changeKey) {
    body = (
      <>
        <Text style={text.small}>
          {`New jobs start with the new key. Jobs that run on ${name} now finish with the current key.`}
        </Text>
        <ProviderConnection
          store={store}
          provider={provider}
          reconnect
          autoFocus
          onConnected={done}
          actions={cancel}
        />
      </>
    );
  } else {
    body = (
      <>
        <Account
          provider={provider}
          name={model.name}
          changeKey={model.actions.changeKey}
          locked={locked}
          onChangeKey={() => setMode("change-key")}
          onDisconnect={onDisconnect}
        />
        {limit === null ? null : (
          <View style={styles.limit}>
            <LimitSection
              store={store}
              provider={provider}
              limitCents={limit}
            />
          </View>
        )}
      </>
    );
  }
  return (
    <Panel title={name} titleRef={title}>
      {body}
    </Panel>
  );
}

const styles = StyleSheet.create({
  spread: {
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "space-between",
    flexWrap: "wrap",
    gap: 12,
  },
  stacked: { flexDirection: "column", alignItems: "flex-start" },
  spreadText: { flexShrink: 1, gap: 4, minWidth: 0 },
  row: { flexDirection: "row", flexWrap: "wrap", gap: 8 },
  limit: {
    paddingTop: 16,
    borderTopWidth: 1,
    borderTopColor: colors.border,
  },
});
