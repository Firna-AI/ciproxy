// The Boat panel of Settings under Overflow: the connected account with
// Change key and Disconnect, or the form of a connection. Every connection
// asks for the key and the monthly limit. A reconnect after Boat rejected
// the key, and a change of the key, start the limit with the saved limit.
// The panel shows the account name and never the key.

import { useState, type RefObject } from "react";
import { StyleSheet, Text, View } from "react-native";

import { ActionButton } from "../../components/actions.tsx";
import { Panel } from "../../components/panel.tsx";
import { text } from "../../components/text.ts";
import { useViewport } from "../../components/viewport.ts";
import type { OverflowSettings } from "../../api/schema.ts";
import type { OverflowStore } from "../../state/overflow.ts";
import { ConnectFormView } from "../boat/connect-form.tsx";
import { useConnectForm } from "../boat/use-connect-form.ts";

const intro =
  "When your machines are busy, ciproxy starts jobs in your own Boat account. Boat bills you by the second.";

const changeNote =
  "New jobs start with the new key. Jobs that run on Boat now finish with the current key.";

function Connection(props: {
  store: OverflowStore;
  savedLimit: string | null;
  reconnect: boolean;
  autoFocus?: boolean;
  onConnected: () => void;
  onCancel?: () => void;
}) {
  const form = useConnectForm(props.store, props.savedLimit, props.onConnected);
  const { onCancel } = props;
  return (
    <ConnectFormView
      form={form}
      reconnect={props.reconnect}
      autoFocus={props.autoFocus}
      actions={
        onCancel ? (
          <ActionButton label="Cancel" tone="ghost" onPress={onCancel} />
        ) : null
      }
    />
  );
}

function Account({
  name,
  onChangeKey,
  onDisconnect,
}: {
  name: string;
  onChangeKey: () => void;
  onDisconnect: () => void;
}) {
  const mobile = useViewport() === "mobile";
  return (
    <View style={[styles.account, mobile && styles.accountStacked]}>
      <View style={styles.accountText}>
        <Text style={[text.body, text.strong]}>
          Connected to the account {name}
        </Text>
        <Text style={text.caption}>
          ciproxy starts overflow jobs in this account. Boat bills it.
        </Text>
      </View>
      <View style={styles.row}>
        <ActionButton label="Change key" size="sm" onPress={onChangeKey} />
        <ActionButton
          label="Disconnect"
          size="sm"
          tone="danger"
          onPress={onDisconnect}
        />
      </View>
    </View>
  );
}

export function BoatPanel({
  store,
  settings,
  titleRef,
  onChanged,
  onDisconnect,
}: {
  store: OverflowStore;
  settings: OverflowSettings;
  titleRef: RefObject<View | null>;
  /** The panel changed in place; focus returns to its title. */
  onChanged: () => void;
  onDisconnect: () => void;
}) {
  const [changing, setChanging] = useState(false);
  const account = settings.account ?? null;
  const savedLimit = settings.spend_limit_cents ?? null;
  const done = () => {
    setChanging(false);
    onChanged();
  };
  let body;
  if (account === null) {
    body = (
      <>
        <Text style={text.small}>{intro}</Text>
        <Connection
          store={store}
          savedLimit={savedLimit}
          reconnect={false}
          onConnected={onChanged}
        />
      </>
    );
  } else if (account.rejected_at) {
    body = (
      <Connection
        store={store}
        savedLimit={savedLimit}
        reconnect
        onConnected={onChanged}
      />
    );
  } else if (changing) {
    body = (
      <>
        <Text style={text.small}>{changeNote}</Text>
        <Connection
          store={store}
          savedLimit={savedLimit}
          reconnect
          autoFocus
          onConnected={done}
          onCancel={done}
        />
      </>
    );
  } else {
    body = (
      <Account
        name={account.name}
        onChangeKey={() => setChanging(true)}
        onDisconnect={onDisconnect}
      />
    );
  }
  return (
    <Panel title="Boat" titleRef={titleRef}>
      {body}
    </Panel>
  );
}

const styles = StyleSheet.create({
  account: {
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "space-between",
    flexWrap: "wrap",
    gap: 12,
  },
  accountStacked: { flexDirection: "column", alignItems: "flex-start" },
  accountText: { flexShrink: 1, gap: 4, minWidth: 0 },
  row: { flexDirection: "row", flexWrap: "wrap", gap: 8 },
});
