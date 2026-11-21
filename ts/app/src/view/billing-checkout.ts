// The review of a plan choice before a payment, from the review that the API
// priced: the plan, its add-ons, the monthly total, what Stripe charges today,
// and the day of each later charge. A move from Free goes on to Stripe
// Checkout; a move between paid plans charges the card on file at once.

import type {
  CheckoutReviewPage,
  PlanTier,
  PlanUsagePage,
} from "../api/billing-types.ts";
import { failureStatus, type ApiFailure } from "../api/client.ts";
import { statusWaits } from "../state/billing-page.ts";
import {
  capacityText,
  formatAddOn,
  lastDayText,
  monthlyText,
  ordinalDay,
  tierNames,
} from "./billing-format.ts";
import { formatCents } from "./exact-format.ts";

export type SummaryRow = { label: string; value: string; strong: boolean };

export type ReviewModel = {
  title: string;
  subtitle: string;
  rows: readonly SummaryRow[];
  note: string;
  submit: string;
};

/** "Move to Team": the title of the review of a tier. */
export function reviewTitle(tier: PlanTier): string {
  return `Move to ${tierNames[tier]}`;
}

export type ReviewFailure = { title: string; body: string; retry: boolean };

/** The words of a choice that has no price to pay, with Try again. */
export const unpricedReview: Readonly<ReviewFailure> = {
  title: "Unable to price this change",
  body: "The price of this choice did not load. Try again.",
  retry: true,
};

/**
 * Why the API priced no review. A conflict means a payment or an operation
 * runs, or the choice is the current plan; Plan and usage tells which.
 */
export function reviewFailure(
  failure: ApiFailure,
  usage: PlanUsagePage | null,
  tier: PlanTier,
): ReviewFailure {
  const status = failureStatus(failure);
  if (status === 409 && usage !== null) {
    if (statusWaits(usage.status)) {
      return {
        title: "A payment or a billing change is in progress",
        body: "Plan and usage shows it until it ends. Jobs keep running.",
        retry: false,
      };
    }
    if (usage.status.selection.tier === tier) {
      return {
        title: `${tierNames[tier]} is your current plan`,
        body: "Plan and usage changes its add-ons.",
        retry: false,
      };
    }
  }
  if (status === 400) {
    return {
      title: "This plan has no review",
      body: "Choose a paid plan on the Plans page.",
      retry: false,
    };
  }
  return { ...unpricedReview };
}

export function reviewModel(
  page: CheckoutReviewPage,
  workspace: string,
): ReviewModel {
  const { choice } = page;
  const { tier, add_ons: addOns } = choice.selection;
  const name = tierNames[tier];
  const today = formatCents(page.today_cents);
  const day = ordinalDay(page.billing_day);
  const row = (label: string, value: string, strong = false): SummaryRow => ({
    label,
    value,
    strong,
  });
  return {
    title: reviewTitle(tier),
    subtitle: `For ${workspace}: ${capacityText(choice)}.`,
    rows: [
      row("Plan", name),
      row("Extra starts", formatAddOn("starts", addOns.start_blocks)),
      row("Extra own machines", formatAddOn("machines", addOns.machines)),
      row("Total a month", monthlyText(choice.monthly_cents), true),
      row("Charged today", today),
    ],
    note: page.prorated
      ? `${name} starts now. Stripe charges the card on file ${today} for the rest of this period, which ends on ${lastDayText(page.period_end)}, then ${formatCents(choice.monthly_cents)} on the ${day} of each month. The starts of this period count toward ${name}.`
      : `Stripe charges ${today} today, then on the ${day} of each month. You pay by card on Stripe, and ciproxy never sees your card.`,
    submit: page.prorated ? `Move to ${name}` : "Continue to payment",
  };
}
