// Plan and usage: the plan, the use of this period against it, the notices of
// this page, and the add-ons. The page reads the API on each visit, and again
// on an interval while Stripe confirms a payment or an operation runs. Every
// role sees the same page; only an Admin or a Billing person pays.

import { useEffect, useMemo, useState } from "react";

import type { PlanAddOns, PlanUsagePage } from "../../api/billing-types.ts";
import { Notice } from "../../components/notice.tsx";
import { PageHeader, ReadFailure } from "../../components/page-state.tsx";
import { ActionButton } from "../../components/actions.tsx";
import { useServices } from "../../services/context.tsx";
import { useValue } from "../../services/hooks.ts";
import { paymentMayStart, usageStore } from "../../state/billing.ts";
import {
  continuablePayment,
  createPagePayment,
  createPortalAction,
} from "../../state/billing-flows.ts";
import { statusWaits } from "../../state/billing-page.ts";
import { paymentBusy } from "../../state/billing-payment.ts";
import { addOnQuote, suggestedAddOns } from "../../view/billing-addons.ts";
import { outcomeModel } from "../../view/billing-outcome.ts";
import { usageModel } from "../../view/billing-usage.ts";
import type { WorkspaceContext } from "../workspace-page.tsx";
import { AddOnsPanel } from "./add-ons-panel.tsx";
import { BillingRoute } from "./billing-route.tsx";
import {
  Half,
  LoadingUsage,
  PanelPair,
  PeriodPanel,
  PlanPanel,
} from "./usage-panels.tsx";
import { UsageNotices } from "./usage-notices.tsx";

function addOnsKey(addOns: PlanAddOns): string {
  return `${addOns.start_blocks}:${addOns.machines}`;
}

function UsageBody({
  page,
  context,
}: {
  page: PlanUsagePage;
  context: WorkspaceContext;
}) {
  const services = useServices();
  const { workspace } = context;
  const portal = useMemo(
    () => createPortalAction(services, workspace.id),
    [services, workspace.id],
  );
  const payment = useMemo(
    () => createPagePayment(services, workspace.id),
    [services, workspace.id],
  );
  const portalState = useValue(portal.state);
  const paymentState = useValue(payment.state);
  const saved = page.status.selection.add_ons;
  const [edited, setEdited] = useState<PlanAddOns>(saved);
  const savedKey = addOnsKey(saved);
  // The edit follows the saved add-ons whenever a save changes them, and
  // keeps the person's quantities through a read that changes nothing.
  useEffect(() => {
    setEdited(page.status.selection.add_ons);
  }, [savedKey]);
  const usageState = useValue(usageStore(services, workspace.id).state);
  const resume = continuablePayment(
    services,
    workspace.id,
    usageState,
    (read) => read.status,
  );
  const model = usageModel(page, workspace.name, resume);
  // The add-ons panel shows the outcome of its own payment; the notice of a
  // pending payment then says nothing more.
  const ownOutcome =
    paymentState.status === "settled" &&
    (paymentState.outcome.state === "open" ||
      paymentState.outcome.state === "payment_required");
  const notices = ownOutcome
    ? model.notices.filter((notice) => notice.kind !== "confirming")
    : model.notices;
  const busy = paymentBusy(paymentState);
  const openPortal = () => void portal.run();
  const save = () => {
    const quote = addOnQuote(page, edited);
    if (quote.kind === "offer") {
      void payment.submit({ kind: "add-ons", quote: quote.offer });
    }
  };
  const suggested = suggestedAddOns(page);
  return (
    <>
      <UsageNotices
        notices={notices}
        workspaceId={workspace.id}
        canPay={page.status.can_pay}
        onAddOns={() => {
          if (suggested !== null) setEdited(suggested);
        }}
        onPortal={openPortal}
      />
      <PanelPair>
        <Half>
          <PlanPanel
            model={model}
            workspaceId={workspace.id}
            canPay={page.status.can_pay}
            portal={portalState}
            onPortal={openPortal}
          />
        </Half>
        <Half>
          <PeriodPanel model={model} />
        </Half>
      </PanelPair>
      <AddOnsPanel
        page={page}
        edited={edited}
        onEdit={setEdited}
        canSave={!busy && paymentMayStart(services, workspace.id)}
        saving={busy}
        outcome={outcomeModel(paymentState, workspace)}
        onSave={save}
        onRetry={() => void payment.retry()}
        onPortal={openPortal}
      />
    </>
  );
}

function PlanUsage({ context }: { context: WorkspaceContext }) {
  const services = useServices();
  const store = usageStore(services, context.workspace.id);
  const snapshot = useValue(store.state);
  useEffect(() => {
    void store.load();
  }, [store]);
  const waiting =
    snapshot.status === "ready" && statusWaits(snapshot.page.status);
  useEffect(() => (waiting ? store.watch() : undefined), [store, waiting]);
  return (
    <>
      <PageHeader title="Plan and usage" />
      {snapshot.status === "loading" ? <LoadingUsage /> : null}
      {snapshot.status === "failed" ? (
        <ReadFailure
          subject="plan and usage"
          onRetry={() => void store.load()}
        />
      ) : null}
      {snapshot.status === "ready" && snapshot.refreshFailure ? (
        <Notice
          tone="danger"
          title="Unable to refresh plan and usage"
          actions={
            <ActionButton
              label="Try again"
              size="sm"
              onPress={() => void store.load()}
            />
          }
        >
          This page shows the last plan that loaded. Payments wait until it
          loads again.
        </Notice>
      ) : null}
      {snapshot.status === "ready" ? (
        <UsageBody page={snapshot.page} context={context} />
      ) : null}
    </>
  );
}

export function PlanUsageScreen() {
  return (
    <BillingRoute path="/billing" stripeReturn="portal">
      {(context) => <PlanUsage context={context} />}
    </BillingRoute>
  );
}
