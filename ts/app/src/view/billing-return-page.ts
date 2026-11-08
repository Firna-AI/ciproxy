// The return page after Stripe Checkout, from the server's answer for the
// exact session only. The page never says that a payment succeeded before
// the server confirms it. After the confirmation, Plan and usage gives the
// plan, its capacity, and its new period. A payment that did not finish, or
// a Checkout that is still open, shows the warning look of the mockup. After
// a confirmed payment, only an Admin gets Add machines, because only an Admin
// adds machines.

import type { PaymentOutcome, PlanUsagePage } from "../api/billing-types.ts";
import { failureStatus } from "../api/client.ts";
import type { WorkspaceRole } from "../api/schema.ts";
import { workspaceHref } from "../routing/pages.ts";
import type { ReturnSnapshot } from "../state/billing-return.ts";
import { askPayer, capacityText, tierNames } from "./billing-format.ts";
import { billingHref, plansHref } from "./billing-routes.ts";
import { usageModel, type MeterModel } from "./billing-usage.ts";

export type ReturnAction =
  | { kind: "link"; label: string; href: string; primary: boolean }
  | { kind: "stripe"; label: string; href: string; newTab: boolean }
  | { kind: "retry"; label: string };

/** The look of a result: done, a warning, plain, or a failure. */
export type ReturnTone = "done" | "warning" | "plain" | "danger";

export type ReturnView =
  | { kind: "waiting"; still: boolean; title: string; body: string }
  | {
      kind: "result";
      tone: ReturnTone;
      title: string;
      body: string;
      actions: readonly ReturnAction[];
      meter: { model: MeterModel; detail: string } | null;
    };

type Workspace = { id: string; name: string };

function settled(
  outcome: PaymentOutcome,
  usage: PlanUsagePage | null,
  workspace: Workspace,
  role: WorkspaceRole,
): ReturnView {
  const usageLink: ReturnAction = {
    kind: "link",
    label: "Open plan and usage",
    href: billingHref(workspace.id),
    primary: false,
  };
  const plans: ReturnAction = {
    kind: "link",
    label: "Back to plans",
    href: plansHref(workspace.id),
    primary: false,
  };
  const result = (
    tone: ReturnTone,
    title: string,
    body: string,
    actions: readonly ReturnAction[],
  ): ReturnView => ({
    kind: "result",
    tone,
    title,
    body,
    actions,
    meter: null,
  });
  switch (outcome.state) {
    case "confirming":
      return { kind: "waiting", still: false, title: "", body: "" };
    case "confirmed": {
      const paid =
        usage !== null && usage.status.selection.tier !== "free" ? usage : null;
      // Only an Admin adds machines, so Billing keeps only Plan and usage.
      const next: readonly ReturnAction[] =
        role === "admin"
          ? [
              {
                kind: "link",
                label: "Add machines",
                href: workspaceHref("machines", workspace.id),
                primary: true,
              },
              usageLink,
            ]
          : [usageLink];
      if (paid === null) {
        return result(
          "done",
          "Stripe confirmed your payment",
          "Plan and usage shows the new plan. Stripe emails the receipt.",
          next,
        );
      }
      const model = usageModel(paid, workspace.name);
      return {
        kind: "result",
        tone: "done",
        title: `${workspace.name} is on ${tierNames[paid.status.selection.tier]}`,
        body: `${capacityText(paid.current)}. Stripe emails the receipt.`,
        actions: next,
        meter: {
          model: { ...model.starts, label: "Starts this period" },
          detail: `The ${tierNames[paid.status.selection.tier]} period runs ${model.period}.`,
        },
      };
    }
    case "unfinished":
      return result(
        "warning",
        "Your plan did not change",
        `Stripe took no payment. ${workspace.name} is still on Free, and jobs keep running.`,
        [{ ...plans, label: "Try again", primary: true }, usageLink],
      );
    case "open":
      return result(
        "warning",
        "Your payment is not finished",
        `Stripe took no payment yet. ${workspace.name} is still on Free, and jobs keep running.`,
        [
          {
            kind: "stripe",
            label: "Continue payment",
            href: outcome.url,
            newTab: false,
          },
          plans,
        ],
      );
    case "payment_required":
      return outcome.payment_url
        ? result(
            "plain",
            "Your payment is not finished",
            "Finish it on its Stripe page. Your plan changes when Stripe confirms the payment.",
            [
              {
                kind: "stripe",
                label: "Open the Stripe page",
                href: outcome.payment_url,
                newTab: true,
              },
              usageLink,
            ],
          )
        : result(
            "plain",
            "Your payment is not finished",
            "Your plan changes when Stripe confirms the payment.",
            [usageLink],
          );
  }
}

export function returnView(
  snapshot: ReturnSnapshot,
  usage: PlanUsagePage | null,
  workspace: Workspace,
  /** The role of the latest session read. Only an Admin adds machines. */
  role: WorkspaceRole,
): ReturnView {
  if (snapshot.status === "confirming") {
    return snapshot.still
      ? {
          kind: "waiting",
          still: true,
          title: "Stripe is still confirming your payment",
          body: `${workspace.name} moves to its new plan when Stripe confirms it. This page updates by itself, and jobs keep running.`,
        }
      : {
          kind: "waiting",
          still: false,
          title: "Confirming your payment",
          body: "Stripe is confirming the payment. This takes a few seconds.",
        };
  }
  if (snapshot.status === "failed") {
    const usageLink: ReturnAction = {
      kind: "link",
      label: "Open plan and usage",
      href: billingHref(workspace.id),
      primary: false,
    };
    return failureStatus(snapshot.failure) === 403
      ? {
          kind: "result",
          tone: "danger",
          title: "This payment is not for this workspace",
          body: `Open the workspace that paid. ${askPayer} for help.`,
          actions: [usageLink],
          meter: null,
        }
      : {
          kind: "result",
          tone: "danger",
          title: "Unable to check the payment",
          body: "Check the connection, then try again. Jobs keep running.",
          actions: [{ kind: "retry", label: "Try again" }, usageLink],
          meter: null,
        };
  }
  return settled(snapshot.outcome, usage, workspace, role);
}
