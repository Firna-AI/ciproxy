// The summary of Checkout, from the review, the one payment gate, and the
// payment of the page. Each state that keeps a payment from starting says why
// and offers what recovers it. A failed Plan and usage read comes first, also
// before the words of a refused review, because without it the page cannot
// tell why the review was refused. Its Try again reads Plan and usage again,
// and the review too when that read failed. When the API refused the quote of
// a paid review, the summary asks for a new review only after the Plan and
// usage read that follows the refusal. Review again only reads, so nothing
// pays until the next click. The priced summary shows only when a payment may
// start, with the one submission that its button sends.

import type { PlanUsagePage } from "../api/billing-types.ts";
import {
  isReviewRequired,
  planSubmission,
  type PaymentSubmission,
} from "../api/billing-payment.ts";
import type { ApiFailure } from "../api/client.ts";
import type { PaymentGate } from "../state/billing.ts";
import type { PaymentSnapshot } from "../state/billing-payment.ts";
import {
  reviewedChoice,
  type ReviewSnapshot,
} from "../state/billing-review.ts";
import {
  reviewFailure,
  reviewModel,
  unpricedReview,
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
  | { kind: "usage"; label: string }
  /** Read a new review after the API refused the quote. It pays nothing. */
  | { kind: "review-again"; label: string };

export type SummaryState =
  | { kind: "loading"; text: string }
  | {
      kind: "blocked";
      title: string;
      body: string;
      action: SummaryAction | null;
    }
  | {
      kind: "ready";
      model: ReviewModel;
      prorated: boolean;
      /** The one submission that the pay button sends. */
      submission: PaymentSubmission;
    };

const blocked = (
  title: string,
  body: string,
  action: SummaryAction | null,
): SummaryState => ({ kind: "blocked", title, body, action });

/**
 * The failure of a plan payment whose quote the API refused, once the Plan
 * and usage read after it ended. Null for every other payment state.
 */
function refusedQuote(payment: PaymentSnapshot): ApiFailure | null {
  if (payment.status !== "failed" || payment.reading) return null;
  if (payment.submission.kind !== "plan") return null;
  return isReviewRequired(payment.failure) ? payment.failure : null;
}

export function checkoutSummary(input: {
  review: ReviewSnapshot;
  gate: PaymentGate;
  /** The latest Plan and usage page, which explains a refused review. */
  usage: PlanUsagePage | null;
  workspace: string;
  /** The payment of this page. A refused quote shows after its fresh read. */
  payment: PaymentSnapshot;
}): SummaryState {
  const { review, gate, usage } = input;
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
  const refused = refusedQuote(input.payment);
  if (refused !== null) {
    const { tier } = review.wanted;
    if (usage !== null && usage.status.selection.tier === tier) {
      // The words that a new review of the current plan would give.
      const current = reviewFailure(refused, usage, tier);
      return blocked(current.title, current.body, null);
    }
    return blocked(
      "This review is out of date",
      "The payment did not start. Review the change again to see its current charge.",
      { kind: "review-again", label: "Review again" },
    );
  }
  const shown = reviewedChoice(review);
  if (shown !== null) {
    const submission = planSubmission(shown);
    if (submission === null) {
      // A paid review without its quote cannot start a payment.
      return blocked(unpricedReview.title, unpricedReview.body, {
        kind: "retry-review",
        label: "Try again",
      });
    }
    return {
      kind: "ready",
      model: reviewModel(shown, input.workspace),
      prorated: shown.prorated,
      submission,
    };
  }
  if (review.review.status === "failed") {
    const failure = reviewFailure(
      review.review.failure,
      usage,
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
