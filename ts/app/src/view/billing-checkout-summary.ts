// The summary of Checkout, from the review and the one payment gate. Each
// state that keeps a payment from starting says why and offers what recovers
// it. A failed Plan and usage read comes first, also before the words of a
// refused review, because without it the page cannot tell why the review was
// refused. Its Try again reads Plan and usage again, and the review too when
// that read failed. The priced summary shows only when a payment may start.

import type { PlanUsagePage } from "../api/billing-types.ts";
import type { PaymentGate } from "../state/billing.ts";
import {
  reviewedChoice,
  type ReviewSnapshot,
} from "../state/billing-review.ts";
import {
  reviewFailure,
  reviewModel,
  type ReviewModel,
} from "./billing-checkout.ts";
import { askPayer } from "./billing-format.ts";

/** What recovers a summary with no price. */
export type SummaryAction =
  /** Read the review again. */
  | { kind: "retry-review"; label: string }
  /** Read Plan and usage again, and the review when it failed too. */
  | { kind: "reload"; label: string }
  /** Open Plan and usage, which shows a payment that waits. */
  | { kind: "usage"; label: string };

export type SummaryState =
  | { kind: "loading"; text: string }
  | {
      kind: "blocked";
      title: string;
      body: string;
      action: SummaryAction | null;
    }
  | { kind: "ready"; model: ReviewModel; prorated: boolean };

const blocked = (
  title: string,
  body: string,
  action: SummaryAction | null,
): SummaryState => ({ kind: "blocked", title, body, action });

export function checkoutSummary(input: {
  review: ReviewSnapshot;
  gate: PaymentGate;
  /** The latest Plan and usage page, which explains a refused review. */
  usage: PlanUsagePage | null;
  workspace: string;
}): SummaryState {
  const { review, gate } = input;
  if (gate === "unknown") {
    return blocked(
      "Unable to load plan and usage",
      "Payments wait until it loads. Jobs keep running.",
      { kind: "reload", label: "Try again" },
    );
  }
  if (gate === "not-payer") {
    return blocked(
      "Only an Admin or a Billing person can pay",
      `${askPayer}.`,
      null,
    );
  }
  if (gate === "waiting") {
    return blocked(
      "A payment or a billing change is in progress",
      "Plan and usage shows it until it ends. Jobs keep running.",
      { kind: "usage", label: "Open plan and usage" },
    );
  }
  if (gate === "loading") {
    return { kind: "loading", text: "Checking your plan" };
  }
  const shown = reviewedChoice(review);
  if (shown !== null) {
    return {
      kind: "ready",
      model: reviewModel(shown, input.workspace),
      prorated: shown.prorated,
    };
  }
  if (review.review.status === "failed") {
    const failure = reviewFailure(
      review.review.failure,
      input.usage,
      review.wanted.tier,
    );
    return blocked(
      failure.title,
      failure.body,
      failure.retry ? { kind: "retry-review", label: "Try again" } : null,
    );
  }
  return { kind: "loading", text: "Pricing this choice" };
}
