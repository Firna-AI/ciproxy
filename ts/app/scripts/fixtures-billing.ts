import type { components } from "../../api/index.ts";

type PlanSelection = components["schemas"]["PlanSelection"];
type PlanOffer = components["schemas"]["PlanOffer"];
type PlanFeature = components["schemas"]["PlanFeature"];

export const planTiers = ["free", "team", "business"] as const;

const published = {
  free: { price: 0n, starts: 2000n, machines: 1n, history: 7 },
  team: { price: 4900n, starts: 25000n, machines: 3n, history: 30 },
  business: { price: 29900n, starts: 150000n, machines: 10n, history: 90 },
};

export function planOffer(selection: PlanSelection): PlanOffer {
  const base = published[selection.tier];
  const blocks = BigInt(selection.add_ons.start_blocks);
  const machines = BigInt(selection.add_ons.machines);
  return {
    selection: structuredClone(selection),
    monthly_cents: String(base.price + blocks * 1000n + machines * 2000n),
    included_starts: String(base.starts + blocks * 10000n),
    included_machines: String(base.machines + machines),
  } satisfies PlanOffer;
}

export function baseSelection(tier: PlanSelection["tier"]): PlanSelection {
  return {
    tier,
    add_ons: { start_blocks: 0, machines: 0 },
  } satisfies PlanSelection;
}

export function historyDays(tier: PlanSelection["tier"]): number {
  return published[tier].history;
}

function feature(name: string, comingLater = false): PlanFeature {
  return { name, coming_later: comingLater } satisfies PlanFeature;
}

function planFeatures(tier: PlanSelection["tier"]): PlanFeature[] {
  return [
    feature("Own Linux machines"),
    feature("Boat overflow"),
    feature("Transparent cache"),
    feature("Warm runners"),
    feature("Savings report"),
    ...(tier === "free"
      ? []
      : [
          feature("Several GitHub organizations"),
          feature("Sticky Docker disks"),
          feature("Email and Slack alerts", true),
          feature("Email support"),
        ]),
    ...(tier !== "business"
      ? []
      : [
          feature("Single sign-on", true),
          feature("Audit log", true),
          feature("Custom VM images", true),
          feature("Priority support"),
        ]),
  ];
}

export function plansPage(
  usage: components["schemas"]["PlanUsagePage"],
): components["schemas"]["PlansPage"] {
  return {
    usage,
    plans: planTiers.map(
      (tier) =>
        ({
          plan: planOffer(baseSelection(tier)),
          current: tier === usage.status.selection.tier,
          can_choose:
            usage.status.can_pay &&
            tier !== usage.status.selection.tier &&
            !usage.status.payment_pending &&
            !usage.status.operation_in_progress,
          covers_use:
            tier === usage.offer.selection.tier &&
            tier !== usage.status.selection.tier,
          history_days: historyDays(tier),
          initial_add_ons:
            tier === usage.offer.selection.tier
              ? structuredClone(usage.offer.selection.add_ons)
              : { start_blocks: 0, machines: 0 },
          organizations: tier === "free" ? 1 : null,
          features: planFeatures(tier),
        }) satisfies components["schemas"]["PlanCard"],
    ),
  } satisfies components["schemas"]["PlansPage"];
}
