// The add-ons of the current paid plan on Plan and usage. The API prices two
// add-on choices on this page: the saved ones, in `current`, and the ones
// that cover the use, in `offer` when it keeps the plan. Any other quantity
// has no price in the API, so the page shows that its total is unknown and
// cannot save it. The page never computes a price itself.

import type {
  PlanAddOns,
  PlanOffer,
  PlanUsagePage,
} from "../api/billing-types.ts";
import { sameSelection } from "../api/billing-payment.ts";
import { capacityText, monthlyText } from "./billing-format.ts";

export type AddOnQuote =
  | { kind: "saved"; offer: PlanOffer }
  | { kind: "offer"; offer: PlanOffer }
  | { kind: "unknown" };

export function addOnQuote(
  page: PlanUsagePage,
  edited: PlanAddOns,
): AddOnQuote {
  const wanted = { tier: page.status.selection.tier, add_ons: edited };
  if (sameSelection(page.current.selection, wanted)) {
    return { kind: "saved", offer: page.current };
  }
  if (sameSelection(page.offer.selection, wanted)) {
    return { kind: "offer", offer: page.offer };
  }
  return { kind: "unknown" };
}

/** The add-ons that cover the use on the current plan, when the API has them. */
export function suggestedAddOns(page: PlanUsagePage): PlanAddOns | null {
  const { offer, current } = page;
  if (offer.selection.tier !== current.selection.tier) return null;
  if (sameSelection(offer.selection, current.selection)) return null;
  return offer.selection.add_ons;
}

export type QuoteText = { total: string; detail: string };

export function quoteText(quote: AddOnQuote): QuoteText {
  if (quote.kind === "unknown") {
    return {
      total: "Monthly total not available",
      detail:
        "Plan and usage prices the saved add-ons and the add-ons that cover your use.",
    };
  }
  return {
    total: monthlyText(quote.offer.monthly_cents),
    detail:
      quote.kind === "saved"
        ? `Your plan with add-ons: ${capacityText(quote.offer)}.`
        : `${capacityText(quote.offer)}. Stripe charges the difference for the rest of this period.`,
  };
}
