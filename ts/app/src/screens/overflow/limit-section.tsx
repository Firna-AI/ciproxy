// The monthly spend limit of one provider, in its panel on Settings under
// Overflow, with the spend of the month: the costs that the provider
// confirmed and the costs that it has not confirmed yet, apart. When the API
// could not read the spend, the section says so, and the limit still applies
// and can change. Until the person types, the field shows the latest saved
// limit. A typed limit stays until its own save is accepted, also when the
// saved limit changes, and a failed save keeps it. The field changes in
// place, so focus stays on Save limit. A save goes to the route of its
// provider, and waits while any overflow change of the workspace runs.

import { useRef, useState } from "react";
import { StyleSheet, Text, View } from "react-native";
import { Input } from "@firna/ui/input";

import { colors } from "@ciproxy/tokens";

import type { ProviderSettings } from "../../api/schema.ts";
import { ActionButton } from "../../components/actions.tsx";
import { AlertIcon } from "../../components/icons.tsx";
import { Notice } from "../../components/notice.tsx";
import { text } from "../../components/text.ts";
import { useValue } from "../../services/hooks.ts";
import type { OverflowStore } from "../../state/overflow.ts";
import { dollarsText, parseDollars } from "../../view/money.ts";
import { spendFigures } from "../../view/overflow.ts";
import {
  limitInvalid,
  limitSaveProblem,
  unsentSave,
  type FieldProblem,
} from "../../view/overflow-failures.ts";
import { providerName } from "../../view/providers.ts";
import { SpendFiguresView } from "./spend-figures.tsx";
import { useSavedDraft } from "./use-saved-draft.ts";

function SpendUnavailable({
  name,
  onRetry,
}: {
  name: string;
  onRetry: () => void;
}) {
  return (
    <View style={styles.missing} role="status">
      <AlertIcon size={16} color={colors.muted} />
      <View style={styles.missingText}>
        <Text style={[text.body, text.strong]}>
          {`Unable to read the ${name} spend`}
        </Text>
        <Text style={text.caption}>The limit still applies.</Text>
      </View>
      <ActionButton label="Try again" size="sm" onPress={onRetry} />
    </View>
  );
}

export function LimitSection({
  store,
  provider,
  limitCents,
}: {
  store: OverflowStore;
  provider: ProviderSettings;
  /** The saved monthly limit of the provider. */
  limitCents: string;
}) {
  const limit = useSavedDraft(dollarsText(limitCents));
  const locked = useValue(store.pending);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [problem, setProblem] = useState<FieldProblem | null>(null);
  const running = useRef(false);
  const figures = spendFigures(provider);
  const fieldError =
    error ??
    (problem !== null && problem.field !== null ? problem.message : null);
  const save = async () => {
    if (running.current) return;
    const amount = parseDollars(limit.text);
    if (!amount.ok) {
      setError(limitInvalid);
      return;
    }
    running.current = true;
    setError(null);
    setProblem(null);
    setBusy(true);
    const result = await store.save({
      provider: provider.provider,
      spend_limit_cents: amount.cents,
    });
    running.current = false;
    setBusy(false);
    const accepted = result !== null && result.ok;
    limit.settle(accepted);
    if (result === null) setProblem({ field: null, copy: unsentSave });
    else if (!result.ok) {
      setProblem(limitSaveProblem(result.failure, provider.provider));
    }
  };
  return (
    <View style={styles.form}>
      <Input
        label="Monthly spend limit, US dollars"
        value={limit.text}
        onChangeText={(value) => {
          limit.edit(value);
          setError(null);
        }}
        onSubmitEditing={() => void save()}
        inputMode="decimal"
        error={fieldError}
        invalid={fieldError !== null}
        editable={!locked}
      />
      {figures === null ? (
        <SpendUnavailable
          name={providerName(provider.provider)}
          onRetry={() => void store.load()}
        />
      ) : (
        <SpendFiguresView figures={figures} />
      )}
      {problem !== null && problem.field === null ? (
        <Notice tone="danger" title={problem.copy.title}>
          {problem.copy.body}
        </Notice>
      ) : null}
      <View style={styles.row}>
        <ActionButton
          label="Save limit"
          tone="primary"
          busy={busy}
          disabled={locked && !busy}
          onPress={() => void save()}
        />
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  form: { gap: 16, maxWidth: 560 },
  row: { flexDirection: "row", flexWrap: "wrap", gap: 8 },
  missing: { flexDirection: "row", alignItems: "flex-start", gap: 10 },
  missingText: { flex: 1, gap: 2, minWidth: 0 },
});
