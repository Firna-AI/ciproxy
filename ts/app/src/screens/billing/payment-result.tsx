// What follows a payment on the page that sent it, from the server's outcome:
// a confirmed plan, a wait for Stripe, a payment to finish on its Stripe
// page, a failed card, a refusal, or a result that the page did not get. Its
// buttons continue the same payment, read Plan and usage again, open the
// Stripe page that the server named, or open the billing portal.

import { View, StyleSheet } from "react-native";
import { Link } from "expo-router";

import { ActionButton, ActionLink } from "../../components/actions.tsx";
import { DoneIcon, ExternalIcon } from "../../components/icons.tsx";
import { Notice } from "../../components/notice.tsx";
import { StateBlock } from "../../components/page-state.tsx";
import type {
  OutcomeAction,
  OutcomeModel,
} from "../../view/billing-outcome.ts";

function ResultAction({
  action,
  onRetry,
  onReload,
  onPortal,
  busy,
}: {
  action: OutcomeAction;
  onRetry: () => void;
  onReload: () => void;
  onPortal: () => void;
  busy: boolean;
}) {
  if (action.kind === "reload") {
    return (
      <ActionButton
        label={action.label}
        size="sm"
        busy={busy}
        onPress={onReload}
      />
    );
  }
  if (action.kind === "retry") {
    return (
      <ActionButton
        label={action.label}
        size="sm"
        busy={busy}
        onPress={onRetry}
      />
    );
  }
  if (action.kind === "portal") {
    return (
      <ActionButton
        label={action.label}
        tone="primary"
        size="sm"
        icon={ExternalIcon}
        onPress={onPortal}
      />
    );
  }
  if (action.newTab || action.href.startsWith("http")) {
    return (
      <ActionLink
        label={action.label}
        href={action.href}
        tone="primary"
        size="sm"
        icon={ExternalIcon}
        newTab={action.newTab}
      />
    );
  }
  return (
    <Link href={action.href} asChild>
      <ActionLink label={action.label} href={action.href} size="sm" />
    </Link>
  );
}

export function PaymentResult({
  model,
  onRetry,
  onReload,
  onPortal,
  busy,
}: {
  model: OutcomeModel;
  onRetry: () => void;
  /** Read Plan and usage again, after that read failed. */
  onReload: () => void;
  onPortal: () => void;
  /** A retry, or the read after the outcome, is on its way. */
  busy: boolean;
}) {
  const actions = model.actions.map((action) => (
    <ResultAction
      key={action.label}
      action={action}
      onRetry={onRetry}
      onReload={onReload}
      onPortal={onPortal}
      busy={busy}
    />
  ));
  if (model.done) {
    return (
      <StateBlock
        icon={DoneIcon}
        title={model.title}
        action={<View style={styles.row}>{actions}</View>}
      >
        {model.body}
      </StateBlock>
    );
  }
  return (
    <Notice tone={model.tone} title={model.title} actions={actions}>
      {model.body}
    </Notice>
  );
}

const styles = StyleSheet.create({
  row: { flexDirection: "row", flexWrap: "wrap", gap: 8 },
});
