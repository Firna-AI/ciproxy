// The summary of a review: the priced choice, the monthly total, what Stripe
// charges today, and the button that pays for exactly this review. While a
// new review loads or after it failed, the summary shows that state and no
// button, so nothing can pay for a choice that the page does not show.

import { StyleSheet, Text, View } from "react-native";
import { Link } from "expo-router";

import { ActionButton, ActionLink } from "../../components/actions.tsx";
import { DetailList, DetailRow } from "../../components/detail-list.tsx";
import { ExternalIcon } from "../../components/icons.tsx";
import { Panel } from "../../components/panel.tsx";
import { WaitingLine } from "../../components/page-state.tsx";
import { text } from "../../components/text.ts";
import type { ReviewModel } from "../../view/billing-checkout.ts";

export type SummaryState =
  | { kind: "loading" }
  | { kind: "failed"; title: string; body: string; retry: boolean }
  | { kind: "ready"; model: ReviewModel; prorated: boolean };

export function CheckoutSummary({
  state,
  plansHref,
  canSubmit,
  submitting,
  onSubmit,
  onRetry,
}: {
  state: SummaryState;
  plansHref: string;
  canSubmit: boolean;
  submitting: boolean;
  onSubmit: () => void;
  onRetry: () => void;
}) {
  const back = (
    <Link href={plansHref} asChild>
      <ActionLink label="Back to plans" href={plansHref} tone="ghost" block />
    </Link>
  );
  if (state.kind === "loading") {
    return (
      <Panel title="Summary">
        <View style={styles.stack}>
          <WaitingLine>Pricing this choice</WaitingLine>
          {back}
        </View>
      </Panel>
    );
  }
  if (state.kind === "failed") {
    return (
      <Panel title="Summary">
        <View style={styles.stack}>
          <View style={styles.tight}>
            <Text style={[text.body, text.strong]}>{state.title}</Text>
            <Text style={text.small}>{state.body}</Text>
          </View>
          {state.retry ? (
            <ActionButton label="Try again" block onPress={onRetry} />
          ) : null}
          {back}
        </View>
      </Panel>
    );
  }
  const { model, prorated } = state;
  return (
    <Panel title="Summary">
      <View style={styles.stack}>
        <DetailList>
          {model.rows.map((row) => (
            <DetailRow key={row.label} label={row.label}>
              {row.strong ? (
                <Text style={[text.body, text.strong]}>{row.value}</Text>
              ) : (
                row.value
              )}
            </DetailRow>
          ))}
        </DetailList>
        <Text style={text.caption}>{model.note}</Text>
        <ActionButton
          label={model.submit}
          tone="primary"
          icon={prorated ? undefined : ExternalIcon}
          block
          busy={submitting}
          disabled={!canSubmit}
          onPress={onSubmit}
        />
        {back}
      </View>
    </Panel>
  );
}

const styles = StyleSheet.create({
  stack: { gap: 14 },
  tight: { gap: 4 },
});
