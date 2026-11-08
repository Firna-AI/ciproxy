// The return from Stripe Checkout. It checks the exact session that Stripe
// returned, waits a few seconds while Stripe confirms, then says that Stripe
// still confirms and updates by itself. The confirmed plan shows only after
// the server confirms it. A payment that did not finish offers to try again.
// Only an Admin or a Billing person follows a payment.

import { useEffect, useMemo } from "react";
import { StyleSheet, Text, View } from "react-native";
import { Link, useLocalSearchParams } from "expo-router";
import type { IconComponent, IconComponentProps } from "@firna/ui";

import { ActionButton, ActionLink } from "../../components/actions.tsx";
import { Meter } from "../../components/billing/meter.tsx";
import {
  AlertIcon,
  DoneIcon,
  ExternalIcon,
  InfoIcon,
} from "../../components/icons.tsx";
import {
  Busy,
  StateBlock,
  type StateTone,
} from "../../components/page-state.tsx";
import { Panel } from "../../components/panel.tsx";
import { text } from "../../components/text.ts";
import { useServices } from "../../services/context.tsx";
import { useValue } from "../../services/hooks.ts";
import { usageStore } from "../../state/billing.ts";
import { createCheckoutReturn } from "../../state/billing-flows.ts";
import { askPayer } from "../../view/billing-format.ts";
import {
  returnView,
  type ReturnAction,
  type ReturnTone,
} from "../../view/billing-return-page.ts";
import { billingHref, sessionIdQuery } from "../../view/billing-routes.ts";
import type { WorkspaceContext } from "../workspace-page.tsx";
import { BillingRoute } from "./billing-route.tsx";

/** A payment card, on the grid of the app icons. */
function CardIcon({
  color,
  size = 16,
  strokeWidth = 1.75,
}: IconComponentProps) {
  return (
    <svg
      aria-hidden="true"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke={color ?? "currentColor"}
      strokeWidth={strokeWidth}
      strokeLinecap="round"
      strokeLinejoin="round"
      style={{ flexShrink: 0 }}
    >
      <rect x="2.5" y="5" width="19" height="14" rx="2" />
      <path d="M2.5 10h19" />
    </svg>
  );
}

/** The icon and the look of each result, as the mockups show them. */
const resultLooks: Readonly<
  Record<ReturnTone, { icon: IconComponent; tone?: StateTone }>
> = {
  done: { icon: DoneIcon, tone: "success" },
  warning: { icon: CardIcon, tone: "warning" },
  plain: { icon: InfoIcon },
  danger: { icon: AlertIcon },
};

function Action({
  action,
  onRetry,
}: {
  action: ReturnAction;
  onRetry: () => void;
}) {
  if (action.kind === "retry") {
    return (
      <ActionButton label={action.label} tone="primary" onPress={onRetry} />
    );
  }
  if (action.kind === "stripe") {
    return (
      <ActionLink
        label={action.label}
        href={action.href}
        tone="primary"
        icon={ExternalIcon}
        newTab={action.newTab}
      />
    );
  }
  return (
    <Link href={action.href} asChild>
      <ActionLink
        label={action.label}
        href={action.href}
        tone={action.primary ? "primary" : "secondary"}
      />
    </Link>
  );
}

function ReturnBody({
  context,
  sessionId,
}: {
  context: WorkspaceContext;
  sessionId: string;
}) {
  const services = useServices();
  const { workspace } = context;
  const store = useMemo(
    () => createCheckoutReturn(services, workspace.id, sessionId),
    [services, workspace.id, sessionId],
  );
  const snapshot = useValue(store.state);
  const usage = useValue(usageStore(services, workspace.id).state);
  useEffect(() => {
    store.start();
    return () => store.stop();
  }, [store]);
  const view = returnView(
    snapshot,
    usage.status === "ready" ? usage.page : null,
    workspace,
  );
  if (view.kind === "waiting") {
    return view.still ? (
      <StateBlock icon={InfoIcon} title={view.title}>
        {view.body}
      </StateBlock>
    ) : (
      <View style={styles.waiting} role="status">
        <Busy size={24} />
        <Text role="heading" aria-level={2} style={text.heading}>
          {view.title}
        </Text>
        <Text style={text.small}>{view.body}</Text>
      </View>
    );
  }
  // The state block lays the buttons out in its own wrapping row.
  const actions = view.actions.map((action) => (
    <Action key={action.label} action={action} onRetry={() => store.retry()} />
  ));
  return (
    <>
      <StateBlock
        icon={resultLooks[view.tone].icon}
        tone={resultLooks[view.tone].tone}
        danger={view.tone === "danger"}
        title={view.title}
        action={actions}
      >
        {view.body}
      </StateBlock>
      {view.meter ? (
        <View style={styles.meter}>
          <Meter model={{ ...view.meter.model, detail: view.meter.detail }} />
        </View>
      ) : null}
    </>
  );
}

function CheckoutReturn({ context }: { context: WorkspaceContext }) {
  const query = useLocalSearchParams<Record<string, string | string[]>>();
  const sessionId = sessionIdQuery(query.session_id);
  const usage = billingHref(context.workspace.id);
  const open = (
    <Link href={usage} asChild>
      <ActionLink label="Open plan and usage" href={usage} tone="primary" />
    </Link>
  );
  return (
    <Panel>
      {context.role === "viewer" ? (
        <StateBlock icon={InfoIcon} title={askPayer} action={open}>
          Only an Admin or a Billing person follows a payment. Plan and usage
          shows the plan.
        </StateBlock>
      ) : sessionId === null ? (
        <StateBlock
          icon={InfoIcon}
          title="This page names no payment"
          action={open}
        >
          Plan and usage shows the plan and any payment that Stripe confirms.
        </StateBlock>
      ) : (
        <ReturnBody context={context} sessionId={sessionId} />
      )}
    </Panel>
  );
}

export function CheckoutReturnScreen() {
  return (
    <BillingRoute path="/checkout-return" stripeReturn="checkout">
      {(context) => <CheckoutReturn context={context} />}
    </BillingRoute>
  );
}

const styles = StyleSheet.create({
  waiting: {
    alignItems: "center",
    gap: 8,
    paddingVertical: 40,
    paddingHorizontal: 16,
  },
  meter: {
    alignSelf: "center",
    width: "100%",
    maxWidth: 380,
    paddingBottom: 32,
  },
});
