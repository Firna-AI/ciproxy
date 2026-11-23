// The provider choice of a first connection, in Setup and in Settings under
// Overflow: a card for each provider of ciproxy, with the picked one checked,
// then the form of the picked provider with its key and its monthly spend
// limit. A provider that the deployment cannot start shows as unavailable
// right now and cannot be picked. The first available provider starts
// picked. A new pick shows a new form, so no typed key passes to another
// provider.

import { useState, type ReactNode } from "react";
import { StyleSheet, Text, View } from "react-native";
import { RadioCard, RadioCardGroup } from "@firna/ui/radio";

import type { OverflowProvider, ProviderSettings } from "../../api/schema.ts";
import { text } from "../../components/text.ts";
import type { OverflowStore } from "../../state/overflow.ts";
import { providerCopy } from "../../view/providers.ts";
import { ProviderFormView } from "./provider-form.tsx";
import { useConnectForm } from "./use-connect-form.ts";

/** The first sentence of a connection, in Setup and in Settings. */
export const connectIntro =
  "When your machines are busy, ciproxy starts jobs in your own account at an overflow provider, which bills you.";

function ProviderChoice({
  providers,
  picked,
  disabled,
  onPick,
}: {
  providers: readonly ProviderSettings[];
  picked: OverflowProvider | null;
  disabled: boolean;
  onPick: (provider: OverflowProvider) => void;
}) {
  return (
    <RadioCardGroup label="Overflow provider">
      {providers.map((item) => {
        const copy = providerCopy[item.provider];
        return (
          <RadioCard
            key={item.provider}
            title={copy.name}
            body={item.available ? copy.summary : "Unavailable right now"}
            checked={item.available && item.provider === picked}
            disabled={!item.available || disabled}
            onPress={() => onPick(item.provider)}
          />
        );
      })}
    </RadioCardGroup>
  );
}

/** The form of one provider, with a new state for each provider. */
export function ProviderConnection({
  store,
  provider,
  reconnect,
  autoFocus,
  onConnected,
  actions,
}: {
  store: OverflowStore;
  provider: ProviderSettings;
  reconnect: boolean;
  autoFocus?: boolean;
  onConnected: () => void;
  /** The actions after the submit button: Skip for now, Cancel, Disconnect. */
  actions?: ReactNode;
}) {
  const form = useConnectForm(store, provider, onConnected);
  return (
    <ProviderFormView
      form={form}
      reconnect={reconnect}
      autoFocus={autoFocus}
      actions={actions}
    />
  );
}

/**
 * The choice of the providers of the settings, then the form of the picked
 * one. Null picks nothing, when no provider is available.
 */
export function ProviderConnect({
  store,
  providers,
  locked,
  onConnected,
  actions,
}: {
  store: OverflowStore;
  providers: readonly ProviderSettings[];
  /** A change of the workspace runs, so the choice waits. */
  locked: boolean;
  /** The API accepted the connection of this provider. */
  onConnected: (provider: OverflowProvider) => void;
  actions?: ReactNode;
}) {
  const first = providers.find((item) => item.available) ?? null;
  const [picked, setPicked] = useState(first?.provider ?? null);
  const provider =
    providers.find((item) => item.provider === picked && item.available) ??
    first;
  return (
    <View style={styles.body}>
      <Text style={text.small}>{connectIntro}</Text>
      <ProviderChoice
        providers={providers}
        picked={provider?.provider ?? null}
        disabled={locked}
        onPick={setPicked}
      />
      {provider === null ? null : (
        <ProviderConnection
          key={provider.provider}
          store={store}
          provider={provider}
          reconnect={false}
          onConnected={() => onConnected(provider.provider)}
          actions={actions}
        />
      )}
    </View>
  );
}

const styles = StyleSheet.create({ body: { gap: 12, maxWidth: 560 } });
