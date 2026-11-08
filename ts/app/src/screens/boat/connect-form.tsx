// The form of every Boat connection, in Setup and in Settings under
// Overflow: the API key, always masked, and the monthly spend limit in US
// dollars, then Connect Boat and the other actions of the place. A failure
// that belongs to neither field shows above the actions.

import type { ReactNode } from "react";
import { StyleSheet, View } from "react-native";
import { Input } from "@firna/ui/input";

import { ActionButton } from "../../components/actions.tsx";
import { Notice } from "../../components/notice.tsx";
import type { ConnectForm } from "./use-connect-form.ts";

const limitHint =
  "At the limit, jobs run on your machines only until the next month.";

export function BoatFields({
  form,
  reconnect,
  autoFocus = false,
}: {
  form: ConnectForm;
  /** A new key for a connection that Boat rejected, or a change of key. */
  reconnect: boolean;
  /** Focus the key field, for a form that a button just opened. */
  autoFocus?: boolean;
}) {
  const submit = () => void form.submit();
  return (
    <>
      <Input
        label={reconnect ? "New Boat API key" : "Boat API key"}
        value={form.apiKey}
        onChangeText={form.setApiKey}
        onSubmitEditing={submit}
        placeholder="boat_live_…"
        hint={
          form.keyError === null
            ? "Create a key in the Boat console, under Settings."
            : undefined
        }
        error={form.keyError}
        invalid={form.keyError !== null}
        secureTextEntry
        autoFocus={autoFocus}
        autoComplete="off"
        autoCapitalize="none"
        autoCorrect={false}
        spellCheck={false}
        editable={!form.busy}
      />
      <Input
        label="Monthly spend limit, US dollars"
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
        editable={!form.busy}
      />
    </>
  );
}

export function ConnectFormView({
  form,
  reconnect,
  autoFocus,
  actions,
}: {
  form: ConnectForm;
  reconnect: boolean;
  autoFocus?: boolean;
  /** The actions after Connect Boat, such as Skip for now or Cancel. */
  actions?: ReactNode;
}) {
  return (
    <View style={styles.form}>
      <BoatFields form={form} reconnect={reconnect} autoFocus={autoFocus} />
      {form.failure ? (
        <Notice tone="danger" title={form.failure.title}>
          {form.failure.body}
        </Notice>
      ) : null}
      <View style={styles.row}>
        <ActionButton
          label="Connect Boat"
          tone="primary"
          busy={form.busy}
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
