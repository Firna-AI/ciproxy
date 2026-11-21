import type { Scenario } from "./harness-scenarios.ts";
import { baseSelection } from "./fixtures-billing.ts";
import { refreshUsage, usageFixture } from "./fixtures-billing-usage.ts";
import { settlePayment } from "./harness-pages-state.ts";

export const pageScenarios = [
  "m21-jobs-live",
  "m21-jobs-quiet",
  "m21-jobs-waiting-unknown",
  "m21-jobs-nothing-fits",
  "m21-jobs-fit-unknown",
  "m21-jobs-request-unknown",
  "m21-jobs-spend-limit",
  "m21-jobs-boat-refused",
  "m21-jobs-overflow-unknown",
  "m21-jobs-history",
  "m21-jobs-history-empty",
  "m21-jobs-detail",
  "m21-jobs-detail-not-found",
  "m21-jobs-live-error",
  "m21-jobs-history-error",
  "m21-jobs-detail-error",
  "m21-savings-report",
  "m21-savings-missing-costs",
  "m21-savings-negative",
  "m21-savings-missing-price",
  "m21-savings-ambiguous-price",
  "m21-savings-this-month",
  "m21-savings-last-month",
  "m21-savings-empty",
  "m21-savings-refresh-incomplete",
  "m21-savings-error",
  "m21-usage-free",
  "m21-usage-paid",
  "m21-usage-business",
  "m21-usage-soft-starts",
  "m21-usage-soft-machines",
  "m21-usage-soft-organizations",
  "m21-usage-near-limit",
  "m21-usage-payment-failed",
  "m21-usage-ended",
  "m21-usage-pending",
  "m21-usage-scheduled-free",
  "m21-usage-operation-pending",
  "m21-usage-error",
  "m21-plans-free",
  "m21-plans-paid",
  "m21-plans-viewer",
  "m21-plans-pending",
  "m21-plans-error",
  "m21-review-free",
  "m21-review-paid",
  "m21-review-changed",
  "m21-review-loading",
  "m21-review-error",
  "m21-review-same-tier-conflict",
  "m21-payment-open",
  "m21-payment-confirming",
  "m21-payment-confirmed",
  "m21-payment-unfinished",
  "m21-payment-required",
  "m21-payment-error",
  "m21-payment-return-error",
  "m21-portal",
  "m21-portal-error",
  "m21-move-to-free",
  "m21-move-to-free-error",
  "m21-banner-inactive",
  "m21-banner-error",
  "m21-viewer",
  "m21-billing",
  "m21-no-role",
] as const;

export function applyPageScenario(scenario: Scenario): void {
  const pages = scenario.pages;
  const name = scenario.name;
  if (name === "soft-limit") pages.usage = usageFixture("over-organizations");
  if (!name.startsWith("m21-")) return;
  pages.jobs = "live";
  pages.usage = usageFixture("paid");
  if (
    name.startsWith("m21-payment-") ||
    name === "m21-plans-free" ||
    name === "m21-review-free" ||
    name === "m21-review-changed" ||
    name === "m21-review-loading" ||
    name === "m21-review-error"
  ) {
    pages.usage = usageFixture("free");
  }
  switch (name) {
    case "m21-jobs-quiet":
      pages.jobs = "quiet";
      break;
    case "m21-jobs-waiting-unknown":
      pages.jobs = "unknown-waiting";
      break;
    case "m21-jobs-nothing-fits":
      pages.jobs = "nothing-fits";
      break;
    case "m21-jobs-fit-unknown":
      pages.jobs = "unknown-fit";
      break;
    case "m21-jobs-request-unknown":
      pages.jobs = "unknown-request";
      break;
    case "m21-jobs-spend-limit":
      pages.jobs = "spend-limit";
      break;
    case "m21-jobs-boat-refused":
      pages.jobs = "boat-refused";
      break;
    case "m21-jobs-overflow-unknown":
      pages.jobs = "overflow-unknown";
      break;
    case "m21-jobs-history-empty":
      pages.history = "empty";
      break;
    case "m21-jobs-detail-not-found":
      pages.detail = "not-found";
      break;
    case "m21-jobs-live-error":
      pages.failures = ["jobs/live"];
      break;
    case "m21-jobs-history-error":
      pages.failures = ["jobs/history"];
      break;
    case "m21-jobs-detail-error":
      pages.failures = ["jobs/detail"];
      break;
    case "m21-savings-missing-costs":
      pages.savings = "missing-costs";
      break;
    case "m21-savings-negative":
      pages.savings = "negative";
      break;
    case "m21-savings-missing-price":
      pages.savings = "missing-price";
      break;
    case "m21-savings-ambiguous-price":
      pages.savings = "ambiguous-price";
      break;
    case "m21-savings-this-month":
      pages.range = "this_month";
      break;
    case "m21-savings-last-month":
      pages.range = "last_month";
      break;
    case "m21-savings-empty":
      pages.savings = "empty";
      break;
    case "m21-savings-refresh-incomplete":
      pages.savings = "refresh-incomplete";
      break;
    case "m21-savings-error":
      pages.failures = ["savings"];
      break;
    case "m21-usage-free":
      pages.usage = usageFixture("free");
      break;
    case "m21-usage-business":
      pages.usage = usageFixture("business");
      break;
    case "m21-usage-soft-starts":
      pages.usage = usageFixture("over-starts");
      break;
    case "m21-usage-soft-machines":
      pages.usage = usageFixture("over-machines");
      break;
    case "m21-usage-soft-organizations":
    case "m21-banner-inactive":
      pages.usage = usageFixture("over-organizations");
      pages.inactiveOrganizations = true;
      break;
    case "m21-usage-near-limit":
      pages.usage = usageFixture("near-limit");
      break;
    case "m21-usage-payment-failed":
      pages.usage = usageFixture("payment-failed");
      break;
    case "m21-usage-ended":
      pages.usage = usageFixture("ended");
      break;
    case "m21-usage-pending":
    case "m21-plans-pending":
      pages.usage = usageFixture("pending");
      break;
    case "m21-usage-scheduled-free":
      pages.usage = usageFixture("scheduled-free");
      break;
    case "m21-usage-operation-pending":
      pages.usage = usageFixture("operation-pending");
      break;
    case "m21-usage-error":
      pages.failures = ["billing"];
      break;
    case "m21-plans-error":
      pages.failures = ["plans"];
      break;
    case "m21-review-paid":
      pages.target = baseSelection("business");
      break;
    case "m21-review-same-tier-conflict":
      pages.target = baseSelection("team");
      break;
    case "m21-review-changed":
      pages.usage.starts = "9000";
      pages.usage.expected_starts = "45000";
      pages.usage.machines = "4";
      refreshUsage(pages.usage);
      pages.target = {
        tier: "team",
        add_ons: { start_blocks: 2, machines: 1 },
      };
      break;
    case "m21-review-loading":
      pages.held = ["checkout/review"];
      break;
    case "m21-review-error":
      pages.failures = ["checkout/review"];
      break;
    case "m21-payment-confirming":
      pages.returned = "confirming";
      pages.usage.status.payment_pending = true;
      break;
    case "m21-payment-confirmed":
      pages.returned = "confirmed";
      settlePayment(pages, "confirmed");
      break;
    case "m21-payment-unfinished":
      pages.returned = "unfinished";
      break;
    case "m21-payment-required":
      pages.usage = usageFixture("paid");
      pages.target = baseSelection("business");
      pages.payment = "payment_required";
      pages.returned = "payment_required";
      break;
    case "m21-payment-error":
      pages.failures = ["checkout", "billing/change", "billing/add-ons"];
      break;
    case "m21-payment-return-error":
      pages.failures = ["checkout/return"];
      break;
    case "m21-portal-error":
      pages.failures = ["billing/portal"];
      break;
    case "m21-move-to-free-error":
      pages.failures = ["billing/move-to-free"];
      break;
    case "m21-banner-error":
      pages.failures = ["banner"];
      break;
  }
  if (name === "m21-savings-empty") pages.usage = usageFixture("free");
  if (name !== "m21-payment-required")
    pages.payment =
      pages.usage.status.selection.tier === "free" ? "open" : "confirmed";
  const member = scenario.session.workspaces[0];
  if (member !== undefined) {
    if (name === "m21-viewer" || name === "m21-plans-viewer")
      member.role = "viewer";
    if (name === "m21-billing") member.role = "billing";
    if (name === "m21-no-role") member.role = null;
  }
}
