// The notices of the billing pages as the current API words them. A payment
// that waits shows on Plan and usage and on Plans: with Continue payment when
// this tab saved the Stripe page of that payment, and with no action when it
// did not. The pace notice names the plan that the API offers. Each notice
// says that jobs keep running.

import { ExternalLink } from "lucide-react";

import { Action, LinkAction } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { paths, type CataloguePath } from "../../components/paths.ts";
import {
  apiOffer,
  choiceMonthly,
  choiceName,
  choiceStarts,
} from "../../fixtures/current-api.ts";
import { formatCount } from "../../fixtures/format.ts";
import { workspace } from "../../fixtures/org.ts";
import { tiers } from "../../fixtures/tiers.ts";
import {
  included,
  paceCrossing,
  usageScenes,
  type UsageScene,
} from "../../fixtures/usage.ts";

/**
 * A payment that waits: an open Checkout or an unpaid invoice whose Stripe
 * page this tab saved, or a payment that Stripe confirms.
 */
export type PendingPayment = "checkout" | "invoice" | "confirming";

const pendingWords: Record<PendingPayment, { title: string; body: string }> = {
  checkout: {
    title: "Your payment is not finished",
    body: "Stripe took no payment yet. Your plan changes when Stripe confirms the payment, and jobs keep running.",
  },
  invoice: {
    title: "Your payment is not finished",
    body: "Finish it on its Stripe page. Your plan changes when Stripe confirms the payment, and jobs keep running.",
  },
  confirming: {
    title: "Stripe is confirming a payment",
    body: "Your plan changes when Stripe confirms it. This page updates by itself, and jobs keep running.",
  },
};

/** On Plans, a payment that Stripe confirms says when a plan opens again. */
const plansWaitBody =
  "You can choose a plan again when it ends. This page updates by itself, and jobs keep running.";

/** Continue payment: Checkout in this tab, an invoice page in a new tab. */
function ContinuePayment({ pending }: { pending: "checkout" | "invoice" }) {
  return pending === "checkout" ? (
    <LinkAction
      to={paths.checkoutStripe}
      tone="primary"
      size="sm"
      icon={ExternalLink}
    >
      Continue payment
    </LinkAction>
  ) : (
    <Action tone="primary" size="sm" icon={ExternalLink}>
      Continue payment
    </Action>
  );
}

export function PendingNotice({
  pending,
  page,
}: {
  pending: PendingPayment;
  page: "usage" | "plans";
}) {
  const words = pendingWords[pending];
  const waits = pending === "confirming";
  return (
    <Notice
      tone="info"
      title={words.title}
      actions={waits ? undefined : <ContinuePayment pending={pending} />}
    >
      {waits && page === "plans" ? plansWaitBody : words.body}
    </Notice>
  );
}

/**
 * The day on which the pace passes the plan, with the offer of the API: the
 * add-ons on the same tier, or the plans to compare for another tier.
 */
export function PaceNotice({
  scene,
  plans,
}: {
  scene: UsageScene;
  plans: CataloguePath;
}) {
  const usage = usageScenes[scene];
  const passesOn = paceCrossing(usage);
  if (passesOn === undefined) return null;
  const offer = apiOffer(scene);
  return (
    <Notice
      tone="warning"
      title={`At this pace, ${workspace.name} passes the ${formatCount(included(usage).starts)} starts of ${tiers[usage.tier].name} on ${passesOn}`}
      actions={
        offer.tier === usage.tier ? (
          <Action tone="primary" size="sm">
            Review add-ons
          </Action>
        ) : (
          <LinkAction to={plans} tone="primary" size="sm">
            Compare plans
          </LinkAction>
        )
      }
    >
      Jobs keep running past it. {choiceName(offer)} includes{" "}
      {formatCount(choiceStarts(offer))} starts a month for{" "}
      {choiceMonthly(offer)}.
    </Notice>
  );
}
