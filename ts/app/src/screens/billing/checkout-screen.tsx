// The review of one exact plan choice before a payment. The person sets the
// add-on quantities; the API prices each choice, and the button pays for the
// review on the page only. From Free, the payment goes on to Stripe Checkout;
// between paid plans, Stripe charges the card on file at once. When a payment
// cannot start, the summary says why, from the one payment gate, and its Try
// again reads Plan and usage again. Only an Admin or a Billing person
// reviews; a Viewer reads who can.

import { useEffect, useMemo, useState } from "react";
import { StyleSheet, View, useWindowDimensions } from "react-native";
import { Link, useLocalSearchParams } from "expo-router";

import type { PlanSelection } from "../../api/billing-types.ts";
import { ActionLink } from "../../components/actions.tsx";
import { Quantity } from "../../components/billing/quantity.tsx";
import { DetailHeader } from "../../components/detail-header.tsx";
import { InfoIcon } from "../../components/icons.tsx";
import { StateBlock } from "../../components/page-state.tsx";
import { Panel } from "../../components/panel.tsx";
import { useMounted } from "../../components/viewport.ts";
import { useServices } from "../../services/context.tsx";
import { useValue } from "../../services/hooks.ts";
import { paymentGate, usageStore } from "../../state/billing.ts";
import {
  checkoutReload,
  createPagePayment,
  createPortalAction,
  createReview,
} from "../../state/billing-flows.ts";
import { paymentBusy } from "../../state/billing-payment.ts";
import { reviewedChoice } from "../../state/billing-review.ts";
import { checkoutSummary } from "../../view/billing-checkout-summary.ts";
import { reviewTitle } from "../../view/billing-checkout.ts";
import { askPayer } from "../../view/billing-format.ts";
import { outcomeModel } from "../../view/billing-outcome.ts";
import {
  billingHref,
  checkoutQuery,
  plansHref,
} from "../../view/billing-routes.ts";
import type { WorkspaceContext } from "../workspace-page.tsx";
import { BillingRoute } from "./billing-route.tsx";
import { CheckoutSummary } from "./checkout-summary.tsx";
import { PaymentResult } from "./payment-result.tsx";

/** The review shows two columns on a window wider than this, as the mockup. */
const twoColumnsAbove = 880;

/**
 * Whether the review shows two columns. The export renders with no window,
 * so the first render shows two columns, and the width decides after mount.
 */
function useTwoColumns(): boolean {
  const { width } = useWindowDimensions();
  return useMounted() ? width > twoColumnsAbove : true;
}

function Review({
  context,
  wanted,
}: {
  context: WorkspaceContext;
  wanted: PlanSelection;
}) {
  const services = useServices();
  const { workspace } = context;
  const columns = useTwoColumns();
  const usage = usageStore(services, workspace.id);
  const usageState = useValue(usage.state);
  const review = useMemo(
    () => createReview(services, workspace.id, wanted),
    [services, workspace.id, wanted],
  );
  const payment = useMemo(
    () => createPagePayment(services, workspace.id),
    [services, workspace.id],
  );
  const portal = useMemo(
    () => createPortalAction(services, workspace.id),
    [services, workspace.id],
  );
  const snapshot = useValue(review.state);
  const paying = useValue(payment.state);
  const [reloading, setReloading] = useState(false);
  useEffect(() => {
    void usage.load();
    void review.ensure();
  }, [usage, review]);
  const busy = paymentBusy(paying);
  // The one gate that the payment store also checks before it sends.
  const summary = checkoutSummary({
    review: snapshot,
    gate: paymentGate(services, workspace.id),
    usage: usageState.status === "ready" ? usageState.page : null,
    workspace: workspace.name,
  });
  const outcome = outcomeModel(paying, workspace, usageState);
  const reload = () => {
    setReloading(true);
    void checkoutReload(services, workspace.id, review).finally(() =>
      setReloading(false),
    );
  };
  const edit = (addOns: PlanSelection["add_ons"]) => {
    payment.reset();
    void review.edit(addOns);
  };
  const plans = plansHref(workspace.id);
  return (
    <>
      <DetailHeader
        crumbs={[{ label: "Plans", href: plans }]}
        title={reviewTitle(wanted.tier)}
        subtitle={summary.kind === "ready" ? summary.model.subtitle : undefined}
      />
      <View style={columns ? styles.columns : styles.stack}>
        <View style={columns ? styles.main : undefined}>
          <Panel title="Add-ons">
            <View style={styles.stack}>
              <Quantity
                label="Extra starts"
                caption="Blocks of starts for each month"
                value={snapshot.wanted.add_ons.start_blocks}
                disabled={busy}
                onChange={(value) =>
                  edit({ ...snapshot.wanted.add_ons, start_blocks: value })
                }
              />
              <Quantity
                label="Extra own machines"
                caption="Own machines above the plan"
                value={snapshot.wanted.add_ons.machines}
                disabled={busy}
                onChange={(value) =>
                  edit({ ...snapshot.wanted.add_ons, machines: value })
                }
              />
            </View>
          </Panel>
        </View>
        <View style={columns ? styles.side : styles.stack}>
          <CheckoutSummary
            state={summary}
            plansHref={plans}
            usageHref={billingHref(workspace.id)}
            canSubmit={summary.kind === "ready" && paying.status === "idle"}
            submitting={busy}
            reloading={reloading}
            onSubmit={() => {
              const shown = reviewedChoice(snapshot);
              if (summary.kind === "ready" && shown !== null) {
                void payment.submit({ kind: "plan", review: shown });
              }
            }}
            onRetryReview={() => void review.retry()}
            onReload={reload}
          />
          {outcome ? (
            <PaymentResult
              model={outcome}
              busy={busy || reloading}
              onRetry={() => void payment.retry()}
              onReload={reload}
              onPortal={() => void portal.run()}
            />
          ) : null}
        </View>
      </View>
    </>
  );
}

function Checkout({ context }: { context: WorkspaceContext }) {
  const query = useLocalSearchParams<Record<string, string | string[]>>();
  // One review for each exact query, so a new query starts a new review.
  const key = [query.tier, query.start_blocks, query.machines].join("|");
  const wanted = useMemo(() => checkoutQuery(query), [key]);
  const plans = plansHref(context.workspace.id);
  const compare = (
    <Link href={plans} asChild>
      <ActionLink label="Compare plans" href={plans} tone="primary" />
    </Link>
  );
  if (context.role === "viewer") {
    return (
      <StateBlock icon={InfoIcon} title={askPayer} action={compare}>
        Only an Admin or a Billing person changes the plan.
      </StateBlock>
    );
  }
  if (wanted === null) {
    return (
      <StateBlock
        icon={InfoIcon}
        title="Choose a plan to review"
        action={compare}
      >
        The Plans page opens the review of each plan.
      </StateBlock>
    );
  }
  return <Review context={context} wanted={wanted} />;
}

export function CheckoutScreen() {
  return (
    <BillingRoute path="/checkout" stripeReturn={null}>
      {(context) => <Checkout context={context} />}
    </BillingRoute>
  );
}

const styles = StyleSheet.create({
  columns: { flexDirection: "row", alignItems: "flex-start", gap: 16 },
  main: { flex: 3, minWidth: 0 },
  side: { flex: 2, minWidth: 0, gap: 16 },
  stack: { gap: 16 },
});
