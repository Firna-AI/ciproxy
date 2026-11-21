import type { components } from "../../api/index.ts";
import { activation, fixtureTime, workspaceId } from "./fixtures-core.ts";
import { baseSelection, planOffer } from "./fixtures-billing.ts";

type PlanUsage = components["schemas"]["PlanUsagePage"];
type Role = components["schemas"]["WorkspaceRole"];

export const usageModes = [
  "free",
  "paid",
  "business",
  "near-limit",
  "over-starts",
  "over-machines",
  "over-organizations",
  "payment-failed",
  "ended",
  "pending",
  "scheduled-free",
  "operation-pending",
] as const;
export type UsageMode = (typeof usageModes)[number];

export function refreshUsage(usage: PlanUsage): void {
  usage.current = planOffer(usage.status.selection);
  usage.starts_over = String(
    maximum(0n, BigInt(usage.starts) - BigInt(usage.current.included_starts)),
  );
  usage.machines_over = String(
    maximum(
      0n,
      BigInt(usage.machines) - BigInt(usage.current.included_machines),
    ),
  );
  usage.organizations_over =
    usage.status.selection.tier === "free"
      ? String(maximum(0n, BigInt(usage.organizations) - 1n))
      : "0";
  const choices = (["team", "business"] as const).map((tier) => {
    const base = planOffer(baseSelection(tier));
    return planOffer({
      tier,
      add_ons: {
        start_blocks: Number(
          maximum(
            0n,
            (BigInt(usage.expected_starts.split(".")[0] ?? "0") -
              BigInt(base.included_starts) +
              9999n) /
              10000n,
          ),
        ),
        machines: Number(
          maximum(0n, BigInt(usage.machines) - BigInt(base.included_machines)),
        ),
      },
    });
  });
  usage.offer = choices.reduce((best, choice) =>
    BigInt(choice.monthly_cents) < BigInt(best.monthly_cents) ? choice : best,
  );
}

function maximum(left: bigint, right: bigint): bigint {
  return left > right ? left : right;
}

export function usageFixture(mode: UsageMode): PlanUsage {
  const free = ["free", "ended", "pending", "over-organizations"].includes(
    mode,
  );
  const selection = baseSelection(
    free ? "free" : mode === "business" ? "business" : "team",
  );
  const usage: PlanUsage = {
    banner: null,
    status: {
      can_pay: true,
      cancel_at: mode === "scheduled-free" ? "2026-12-04T00:00:00Z" : null,
      operation_in_progress: mode === "operation-pending",
      payment_pending: mode === "pending",
      period_start: free ? "2026-11-01T00:00:00Z" : "2026-11-04T00:00:00Z",
      period_end: free ? "2026-12-01T00:00:00Z" : "2026-12-04T00:00:00Z",
      selection,
      subscription: free
        ? mode === "ended"
          ? "ended"
          : "none"
        : mode === "payment-failed"
          ? "payment_failed"
          : "active",
    },
    current: planOffer(selection),
    offer: planOffer(baseSelection("team")),
    starts: free
      ? "1240"
      : mode === "near-limit"
        ? "2590"
        : mode === "over-starts" || mode === "payment-failed"
          ? "25310"
          : "1860",
    expected_starts: free
      ? "6200"
      : mode === "near-limit"
        ? "25900"
        : mode === "over-starts" || mode === "payment-failed"
          ? "253100"
          : "18600",
    machines:
      mode === "over-machines" || mode === "payment-failed"
        ? "4"
        : free
          ? "1"
          : "3",
    organizations:
      mode === "over-organizations" || mode === "ended" ? "2" : "1",
    starts_over: "0",
    machines_over: "0",
    organizations_over: "0",
    crossing_day:
      mode === "near-limit" ? "2026-12-02" : free ? "2026-11-10" : null,
  };
  if (mode === "ended") {
    usage.status.period_start = fixtureTime;
    usage.status.period_end = "2026-12-06T12:00:00Z";
    usage.starts = "380";
    usage.expected_starts = "11400";
    usage.machines = "3";
  }
  refreshUsage(usage);
  return usage;
}

export function pageBanner(
  usage: PlanUsage,
  role: Role,
  inactive = false,
): components["schemas"]["PageBanner"] | null {
  const canPay = role !== "viewer";
  const action = canPay ? "choose_plan" : "ask_admin_or_billing";
  if (usage.status.subscription === "payment_failed")
    return {
      kind: "payment_failed",
      action: canPay ? "update_card" : action,
      failed_at: "2026-11-04T00:00:00Z",
      next_retry_at: "2026-11-07T00:00:00Z",
    };
  if (usage.status.subscription === "ended")
    return {
      kind: "subscription_ended",
      action,
      until: usage.status.period_end,
    };
  if (inactive)
    return {
      kind: "inactive_organizations",
      organizations: [
        {
          id: "40000000-0000-4000-8000-000000000002",
          login: "orbit-labs",
          activation: activation({
            admitted: false,
            reasons: ["billing_upgrade"],
            upgrade: canPay
              ? { tier: "team", workspace_id: workspaceId }
              : null,
            ask_admin: !canPay,
            can_enable: false,
          }),
        },
      ],
    };
  if (
    [usage.starts_over, usage.machines_over, usage.organizations_over].some(
      (value) => BigInt(value) > 0n,
    )
  )
    return {
      kind: "soft_limit",
      action,
      offer: usage.offer,
    };
  return null;
}

export function usageForRole(usage: PlanUsage, role: Role): PlanUsage {
  return {
    ...structuredClone(usage),
    status: { ...structuredClone(usage.status), can_pay: role !== "viewer" },
    banner: pageBanner(usage, role),
  };
}
