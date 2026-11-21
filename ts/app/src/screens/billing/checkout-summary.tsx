// The summary of a review: the priced choice, the monthly total, what Stripe
// charges today, and the button that pays for exactly this review. While a
// read runs, the summary says which. When a payment cannot start, it says why
// and offers what recovers it, with no button that pays, so nothing can pay
// for a choice that the page does not show. After the API refused a quote,
// Review again reads a new review, and only the next click on its button pays.

import { StyleSheet, Text, View } from "react-native";
import { Link } from "expo-router";

import { ActionButton, ActionLink } from "../../components/actions.tsx";
import { DetailList, DetailRow } from "../../components/detail-list.tsx";
import { ExternalIcon } from "../../components/icons.tsx";
import { Panel } from "../../components/panel.tsx";
import { WaitingLine } from "../../components/page-state.tsx";
import { text } from "../../components/text.ts";
import type {
  SummaryAction,
  SummaryState,
} from "../../view/billing-checkout-summary.ts";

function Recover({
  action,
  usageHref,
  reloading,
  onRetryReview,
  onReload,
  onReviewAgain,
}: {
  action: SummaryAction;
  usageHref: string;
  reloading: boolean;
  onRetryReview: () => void;
  onReload: () => void;
  onReviewAgain: () => void;
}) {
  if (action.kind === "review-again") {
    return (
      <ActionButton
        label={action.label}
        tone="primary"
        block
        onPress={onReviewAgain}
      />
    );
  }
  if (action.kind === "reload") {
    return (
      <ActionButton
        label={action.label}
        tone="primary"
        block
        busy={reloading}
        onPress={onReload}
      />
    );
  }
  if (action.kind === "usage") {
    return (
      <Link href={usageHref} asChild>
        <ActionLink label={action.label} href={usageHref} block />
      </Link>
    );
  }
  return <ActionButton label={action.label} block onPress={onRetryReview} />;
}

export function CheckoutSummary({
  state,
  plansHref,
  usageHref,
  canSubmit,
  submitting,
  reloading,
  onSubmit,
  onRetryReview,
  onReload,
  onReviewAgain,
}: {
  state: SummaryState;
  plansHref: string;
  usageHref: string;
  canSubmit: boolean;
  submitting: boolean;
  /** Try again of a failed Plan and usage read is on its way. */
  reloading: boolean;
  onSubmit: () => void;
  onRetryReview: () => void;
  onReload: () => void;
  /** Read a new review after a refused quote. It sends no payment. */
  onReviewAgain: () => void;
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
          <WaitingLine>{state.text}</WaitingLine>
          {back}
        </View>
      </Panel>
    );
  }
  if (state.kind === "blocked") {
    return (
      <Panel title="Summary">
        <View style={styles.stack}>
          <View style={styles.tight}>
            <Text style={[text.body, text.strong]}>{state.title}</Text>
            <Text style={text.small}>{state.body}</Text>
          </View>
          {state.action ? (
            <Recover
              action={state.action}
              usageHref={usageHref}
              reloading={reloading}
              onRetryReview={onRetryReview}
              onReload={onReload}
              onReviewAgain={onReviewAgain}
            />
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
