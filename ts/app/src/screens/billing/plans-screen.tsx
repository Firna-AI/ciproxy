// Plans: the three published plans side by side, with the plan that covers
// the use of this period marked. A plan opens its review; Free on a paid plan
// asks before Move to Free. While Stripe confirms a payment, the page says so
// and opens no second checkout. A Viewer compares the plans and reads who can
// choose. Stripe's cancel URL also opens this page.

import { useEffect, useMemo, useState } from "react";
import { StyleSheet, Text, View } from "react-native";

import { DetailHeader } from "../../components/detail-header.tsx";
import { Notice } from "../../components/notice.tsx";
import { Busy, ReadFailure } from "../../components/page-state.tsx";
import { text } from "../../components/text.ts";
import { useViewport } from "../../components/viewport.ts";
import { useServices } from "../../services/context.tsx";
import { useValue } from "../../services/hooks.ts";
import { plansStore } from "../../state/billing.ts";
import {
  continuablePayment,
  createMoveToFree,
} from "../../state/billing-flows.ts";
import { billingHref } from "../../view/billing-routes.ts";
import { plansModel } from "../../view/billing-plans.ts";
import { pendingNotice } from "../../view/billing-usage.ts";
import type { WorkspaceContext } from "../workspace-page.tsx";
import { BillingRoute } from "./billing-route.tsx";
import { MoveToFreeDialog } from "./move-to-free-dialog.tsx";
import { PlanCard } from "./plan-card.tsx";
import { ContinuePayment } from "./usage-notices.tsx";

function Plans({ context }: { context: WorkspaceContext }) {
  const services = useServices();
  const { workspace } = context;
  const store = plansStore(services, workspace.id);
  const snapshot = useValue(store.state);
  const wide = useViewport() === "wide";
  const [asking, setAsking] = useState(false);
  const moveToFree = useMemo(
    () =>
      createMoveToFree(services, workspace.id, () =>
        services.navigator.go(billingHref(workspace.id)),
      ),
    [services, workspace.id],
  );
  const moving = useValue(moveToFree.state);
  useEffect(() => {
    void store.load();
  }, [store]);
  const model =
    snapshot.status === "ready"
      ? plansModel(snapshot.page, workspace.id)
      : null;
  const waiting = model?.waiting ?? false;
  useEffect(() => (waiting ? store.watch() : undefined), [store, waiting]);
  const resume = continuablePayment(
    services,
    workspace.id,
    snapshot,
    (page) => page.usage.status,
  );
  const pending =
    snapshot.status === "ready"
      ? pendingNotice(snapshot.page.usage.status, resume)
      : null;
  const waitingBody =
    pending?.resume === null &&
    snapshot.status === "ready" &&
    snapshot.page.usage.status.payment_pending
      ? "You can choose a plan again when it ends. This page updates by itself, and jobs keep running."
      : pending?.body;
  return (
    <>
      <DetailHeader
        crumbs={[{ label: "Plan and usage", href: billingHref(workspace.id) }]}
        title="Plans"
        subtitle={model?.use}
      />
      {snapshot.status === "loading" ? <Busy size={24} /> : null}
      {snapshot.status === "failed" ? (
        <ReadFailure subject="the plans" onRetry={() => void store.load()} />
      ) : null}
      {snapshot.status === "ready" && snapshot.refreshFailure ? (
        <Notice tone="danger" title="Unable to refresh the plans">
          This page shows the plans that loaded last. Check the connection.
        </Notice>
      ) : null}
      {pending ? (
        <Notice
          tone="info"
          title={pending.title}
          actions={
            pending.resume ? (
              <ContinuePayment resume={pending.resume} />
            ) : undefined
          }
        >
          {waitingBody}
        </Notice>
      ) : null}
      {model ? (
        <View style={wide ? styles.row : styles.column}>
          {model.cards.map((card) => (
            <PlanCard
              key={card.tier}
              model={card}
              onMoveToFree={() => setAsking(true)}
            />
          ))}
        </View>
      ) : null}
      {model ? (
        <Text style={text.caption}>
          Limits are soft: jobs keep running past them, and ciproxy offers the
          plan or add-ons that cover your use. Prices in US dollars, billed
          monthly through Stripe.
        </Text>
      ) : null}
      {asking && snapshot.status === "ready" ? (
        <MoveToFreeDialog
          page={snapshot.page}
          workspace={workspace.name}
          state={moving}
          onConfirm={() => void moveToFree.run()}
          onClose={() => setAsking(false)}
        />
      ) : null}
    </>
  );
}

export function PlansScreen() {
  return (
    <BillingRoute path="/plans" stripeReturn="checkout">
      {(context) => <Plans context={context} />}
    </BillingRoute>
  );
}

const styles = StyleSheet.create({
  row: { flexDirection: "row", alignItems: "stretch", gap: 16 },
  column: { gap: 16 },
});
