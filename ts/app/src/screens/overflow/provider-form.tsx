// The form of every provider connection, in Setup and in Settings under
// Overflow: the API key of the provider, always masked, and the monthly
// spend limit of that provider in US dollars, then the submit button and the
// other actions of the place. The key field shows the placeholder, hint, and
// error copy of its provider. In the Boat branch, the wallet choice shows
// between the key and the limit after the key check, and the same button
// then reads Connect Boat, so focus stays on it. A failure that belongs to
// no field shows above the actions.

import type { ReactNode } from "react";
import { StyleSheet, View } from "react-native";
import { Input } from "@firna/ui/input";

import { ActionButton } from "../../components/actions.tsx";
import { Notice } from "../../components/notice.tsx";
import { providerCopy } from "../../view/providers.ts";
import { WalletChoice } from "../boat/wallet-choice.tsx";
import type { ConnectForm } from "./use-connect-form.ts";

export function ProviderFields({
  form,
  reconnect,
  autoFocus = false,
}: {
  form: ConnectForm;
  /** A new key for a connection that the provider rejected, or Change key. */
  reconnect: boolean;
  /** Focus the key field, for a form that a button just opened. */
  autoFocus?: boolean;
}) {
  const copy = providerCopy[form.provider.provider];
  const submit = () => void form.submit();
  const limitHint = `At the limit, ciproxy starts no new jobs on ${copy.name} until the next month.`;
  return (
    <>
      <Input
        label={`${reconnect ? "New " : ""}${copy.name} API key`}
        value={form.apiKey}
        onChangeText={form.setApiKey}
        onSubmitEditing={submit}
        placeholder={copy.keyPlaceholder}
        hint={form.keyError === null ? copy.keyHint : undefined}
        error={form.keyError}
        invalid={form.keyError !== null}
        secureTextEntry
        autoFocus={autoFocus}
        autoComplete="off"
        autoCapitalize="none"
        autoCorrect={false}
        spellCheck={false}
        editable={!form.busy && !form.locked}
      />
      {form.wallets === null ? null : (
        <WalletChoice
          options={form.wallets}
          value={form.wallet}
          error={form.walletError}
          disabled={form.busy || form.locked}
          onChange={form.setWallet}
        />
      )}
      <Input
        label={`Monthly spend limit on ${copy.name}, US dollars`}
        value={form.limit}
        onChangeText={form.setLimit}
        onSubmitEditing={submit}
        placeholder="For example, 200"
        inputMode="decimal"
        hint={
          form.limitError === null
            ? form.savedLimit
              ? `Your saved limit. ${limitHint}`
              : limitHint
            : undefined
        }
        error={form.limitError}
        invalid={form.limitError !== null}
        editable={!form.busy && !form.locked}
      />
    </>
  );
}

export function ProviderFormView({
  form,
  reconnect,
  autoFocus,
  actions,
}: {
  form: ConnectForm;
  reconnect: boolean;
  autoFocus?: boolean;
  /** The actions after the submit button, such as Skip for now or Cancel. */
  actions?: ReactNode;
}) {
  return (
    <View style={styles.form}>
      <ProviderFields form={form} reconnect={reconnect} autoFocus={autoFocus} />
      {form.failure ? (
        <Notice tone="danger" title={form.failure.title}>
          {form.failure.body}
        </Notice>
      ) : null}
      <View style={styles.row}>
        <ActionButton
          label={form.submitLabel}
          tone="primary"
          busy={form.busy}
          disabled={form.locked}
          onPress={() => void form.submit()}
        />
        {actions}
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  form: { gap: 12, maxWidth: 560 },
  row: { flexDirection: "row", flexWrap: "wrap", gap: 8 },
});
