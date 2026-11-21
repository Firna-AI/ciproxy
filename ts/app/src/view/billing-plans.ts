// Plans, from the page that the API gives: the three published plans, which
// one is current, which one covers the use, and whether this person may
// choose now. The API decides each of these; this model only words them and
// lists what each plan adds to the plan before it.

import type {
  PlanCard,
  PlanFeature,
  PlanTier,
  PlansPage,
} from "../api/billing-types.ts";
import { statusWaits } from "../state/billing-page.ts";
import { lastDayText, machinesText, tierNames } from "./billing-format.ts";
import { checkoutHref } from "./billing-routes.ts";
import { formatCents, formatCount, formatWhole } from "./exact-format.ts";

export type PlanCardAction =
  | { kind: "current" }
  | { kind: "ask" }
  | {
      kind: "choose";
      label: string;
      href: string;
      primary: boolean;
      enabled: boolean;
    }
  | { kind: "move-to-free"; label: string; primary: boolean; enabled: boolean };

export type PlanFeatureModel = { name: string; comingLater: boolean };

export type PlanCardModel = {
  tier: PlanTier;
  name: string;
  /** "$49". */
  price: string;
  /** "25,000 starts a month · 3 own machines". */
  capacity: string;
  tag: "current" | "covers" | null;
  action: PlanCardAction;
  /** "Free includes", or "Everything in Free, plus". */
  lead: string;
  features: readonly PlanFeatureModel[];
};

export type PlansModel = {
  /** The use of this period, which the plans compare against. */
  use: string;
  /** Stripe confirms a payment or an operation runs: nothing can be chosen. */
  waiting: boolean;
  /** The current plan, for the dialog of Move to Free. */
  currentName: string;
  cards: readonly PlanCardModel[];
};

/**
 * What a plan adds to the plan before it: the features that ship now, its
 * days of job history, then the features that come later.
 */
function addedFeatures(
  card: PlanCard,
  before: PlanCard | undefined,
): PlanFeatureModel[] {
  const earlier = new Set(before?.features.map((item) => item.name) ?? []);
  const added = card.features.filter((item) => !earlier.has(item.name));
  const model = (item: PlanFeature): PlanFeatureModel => ({
    name: item.name,
    comingLater: item.coming_later,
  });
  return [
    ...added.filter((item) => !item.coming_later).map(model),
    { name: `${card.history_days} days of job history`, comingLater: false },
    ...added.filter((item) => item.coming_later).map(model),
  ];
}

export function plansModel(page: PlansPage, workspaceId: string): PlansModel {
  const { usage } = page;
  const { status } = usage;
  const cards = page.plans.map((card, index): PlanCardModel => {
    const tier = card.plan.selection.tier;
    const name = tierNames[tier];
    const before = index === 0 ? undefined : page.plans[index - 1];
    const action: PlanCardAction = card.current
      ? { kind: "current" }
      : !status.can_pay
        ? { kind: "ask" }
        : tier === "free"
          ? {
              kind: "move-to-free",
              label: "Move to Free",
              primary: card.covers_use,
              enabled: card.can_choose,
            }
          : {
              kind: "choose",
              label: `Choose ${name}`,
              href: checkoutHref(workspaceId, {
                tier,
                add_ons: card.initial_add_ons,
              }),
              primary: card.covers_use,
              enabled: card.can_choose,
            };
    return {
      tier,
      name,
      price: formatCents(card.plan.monthly_cents),
      capacity: `${formatCount(card.plan.included_starts)} starts a month · ${machinesText(card.plan.included_machines)}`,
      tag: card.current ? "current" : card.covers_use ? "covers" : null,
      action,
      lead:
        before === undefined
          ? `${name} includes`
          : `Everything in ${tierNames[before.plan.selection.tier]}, plus`,
      features: addedFeatures(card, before),
    };
  });
  return {
    use: `This period so far: ${formatCount(usage.starts)} starts on ${machinesText(usage.machines)}. At this pace, about ${formatWhole(usage.expected_starts)} starts by ${lastDayText(status.period_end)}.`,
    waiting: statusWaits(status),
    currentName: tierNames[status.selection.tier],
    cards,
  };
}
