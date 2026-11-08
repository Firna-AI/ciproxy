// The monthly spend limit, with the spend of the month: the costs that Boat
// confirmed and the costs that it has not confirmed yet, apart. When the API
// could not read the spend, the panel says so, and the limit still applies
// and can change. A failed save keeps the typed limit in the field. A new
// saved limit replaces the field in place, so focus stays on Save.

import { useEffect, useState } from "react";
import { StyleSheet, Text, View } from "react-native";
import { Input } from "@firna/ui/input";

import { colors } from "@ciproxy/tokens";

import { ActionButton } from "../../components/actions.tsx";
import { AlertIcon } from "../../components/icons.tsx";
import { Notice } from "../../components/notice.tsx";
import { Panel } from "../../components/panel.tsx";
import { text } from "../../components/text.ts";
import type { OverflowSettings } from "../../api/schema.ts";
import { spendFigures } from "../../view/boat.ts";
import { limitInvalid } from "../../view/boat-failures.ts";
import { dollarsText, parseDollars } from "../../view/money.ts";
import { SpendFiguresView } from "../boat/spend-figures.tsx";
import type { PolicyEditor } from "./use-policy.ts";

function SpendUnavailable({ onRetry }: { onRetry: () => void }) {
  return (
    <View style={styles.missing} role="status">
      <AlertIcon size={16} color={colors.muted} />
      <View style={styles.missingText}>
        <Text style={[text.body, text.strong]}>
          Unable to read the Boat spend
        </Text>
        <Text style={text.caption}>The limit still applies.</Text>
      </View>
      <ActionButton label="Try again" size="sm" onPress={onRetry} />
    </View>
  );
}

export function SpendPanel({
  settings,
  limitCents,
  editor,
  onRetry,
}: {
  settings: OverflowSettings;
  limitCents: string;
  editor: PolicyEditor;
  /** Read the settings again, for a spend that could not be read. */
  onRetry: () => void;
}) {
  const saved = dollarsText(limitCents);
  const [limit, setLimit] = useState(saved);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    setLimit(saved);
    setError(null);
  }, [saved]);
  const figures = spendFigures(settings);
  const problem = editor.limitProblem;
  const fieldError =
    error ??
    (problem !== null && problem.field !== null ? problem.message : null);
  const save = () => {
    const amount = parseDollars(limit);
    if (!amount.ok) {
      setError(limitInvalid);
      return;
    }
    setError(null);
    void editor.save("limit", { spend_limit_cents: amount.cents });
  };
  return (
    <Panel title="Monthly spend limit">
      <View style={styles.form}>
        <Input
          label="Limit, US dollars"
          value={limit}
          onChangeText={(value) => {
            setLimit(value);
            setError(null);
          }}
          onSubmitEditing={save}
          inputMode="decimal"
          error={fieldError}
          invalid={fieldError !== null}
          editable={editor.saving === null}
        />
        {figures === null ? (
          <SpendUnavailable onRetry={onRetry} />
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
            label="Save"
            tone="primary"
            busy={editor.saving === "limit"}
            disabled={editor.saving === "wait"}
            onPress={save}
          />
        </View>
      </View>
    </Panel>
  );
}

const styles = StyleSheet.create({
  form: { gap: 16, maxWidth: 560 },
  row: { flexDirection: "row", flexWrap: "wrap", gap: 8 },
  missing: { flexDirection: "row", alignItems: "flex-start", gap: 10 },
  missingText: { flex: 1, gap: 2, minWidth: 0 },
});
