import type { components } from "../../api/index.ts";
import { type JobsMode } from "./fixtures-jobs-live.ts";
import { type SavingsMode } from "./fixtures-savings.ts";
import { refreshUsage, usageFixture } from "./fixtures-billing-usage.ts";
import { baseSelection } from "./fixtures-billing.ts";
import { type PaymentState } from "./fixtures-billing-payment.ts";
import { fixtureTime } from "./fixtures-core.ts";
import type { CapturedBody } from "./harness-pages-facts.ts";

type PageBody = components["schemas"][
  | "JobsLivePage"
  | "JobsHistoryPage"
  | "JobDetailPage"
  | "SavingsPage"
  | "PlanUsagePage"
  | "PlansPage"
  | "CheckoutReviewPage"
  | "CheckoutReturnPage"
  | "PaymentOutcome"
  | "PortalLink"
  | "Success"
  | "PageBannerView"
  | "Problem"];
export type PageReply = {
  status: number;
  body: PageBody;
  requestBody?: CapturedBody;
};

export function pageReply(
  body: PageBody,
  requestBody?: CapturedBody,
): PageReply {
  return { status: 200, body, requestBody };
}

export function pageError(
  status: number,
  code: components["schemas"]["ProblemCode"],
  message: string,
  requestBody?: CapturedBody,
): PageReply {
  return { status, body: { code, message }, requestBody };
}

export const pageRoutes = [
  "jobs/live",
  "jobs/history",
  "jobs/detail",
  "savings",
  "billing",
  "plans",
  "checkout/review",
  "checkout",
  "checkout/return",
  "billing/change",
  "billing/add-ons",
  "billing/portal",
  "billing/move-to-free",
  "banner",
] as const;
export type PageRoute = (typeof pageRoutes)[number];

export type PagesFixtureState = {
  jobs: JobsMode;
  history: "populated" | "empty";
  detail: "found" | "not-found";
  savings: SavingsMode;
  range: components["schemas"]["SavingsRange"];
  usage: components["schemas"]["PlanUsagePage"];
  payment: PaymentState;
  returned: PaymentState;
  target: components["schemas"]["PlanSelection"];
  inactiveOrganizations: boolean;
  failures: PageRoute[];
  held: PageRoute[];
  pending: { route: PageRoute; release: () => void }[];
};

export function createPagesFixture(): PagesFixtureState {
  return {
    jobs: "quiet",
    history: "populated",
    detail: "found",
    savings: "report",
    range: "last30_days",
    usage: usageFixture("free"),
    payment: "open",
    returned: "confirming",
    target: baseSelection("team"),
    inactiveOrganizations: false,
    failures: [],
    held: [],
    pending: [],
  };
}

export async function waitForPage(
  state: PagesFixtureState,
  route: PageRoute,
): Promise<void> {
  if (state.held.includes(route)) {
    await new Promise<void>((release) =>
      state.pending.push({ route, release }),
    );
  }
}

export function releasePages(
  state: PagesFixtureState,
  held: PageRoute[] = [],
): void {
  state.held = held;
  const ready = state.pending.filter((item) => !held.includes(item.route));
  state.pending = state.pending.filter((item) => held.includes(item.route));
  ready.forEach((item) => item.release());
}

export function settlePayment(
  state: PagesFixtureState,
  outcome: PaymentState,
): void {
  const usage = state.usage;
  usage.status.payment_pending =
    outcome === "open" ||
    outcome === "confirming" ||
    outcome === "payment_required";
  if (outcome !== "confirmed") return;
  const current = usage.status.selection;
  if (
    current.tier === state.target.tier &&
    current.add_ons.start_blocks === state.target.add_ons.start_blocks &&
    current.add_ons.machines === state.target.add_ons.machines
  )
    return;
  if (usage.status.selection.tier === "free") {
    usage.starts = "0";
    usage.expected_starts = "0";
    usage.crossing_day = null;
    usage.status.period_start = fixtureTime;
    usage.status.period_end = "2026-12-06T12:00:00Z";
    usage.status.cancel_at = null;
  }
  usage.status.selection = structuredClone(state.target);
  usage.status.subscription = "active";
  refreshUsage(usage);
}

export function pagesSnapshot(state: PagesFixtureState) {
  return {
    jobs: state.jobs,
    history: state.history,
    detail: state.detail,
    savings: state.savings,
    range: state.range,
    usage: state.usage,
    payment: state.payment,
    returned: state.returned,
    target: state.target,
    inactiveOrganizations: state.inactiveOrganizations,
    failures: state.failures,
    held: state.held,
    pending: state.pending.map((item) => item.route),
  };
}
