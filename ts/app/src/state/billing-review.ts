// The review of one plan choice before a payment. The person sets the add-on
// quantities; each change asks the API to price the new choice, and the
// summary shows only a review of exactly the wanted choice. While a new
// review loads or after it fails, no review is shown, so nothing can pay for
// a choice that the page does not show. A newer review always wins.

import type { ApiFailure, ApiResult } from "../api/client.ts";
import type {
  CheckoutReviewPage,
  PlanAddOns,
  PlanSelection,
} from "../api/billing-types.ts";
import { sameSelection } from "../api/billing-payment.ts";
import { createStore, type Store } from "./store.ts";

export type ReviewState =
  | { status: "loading" }
  | { status: "ready"; page: CheckoutReviewPage }
  | { status: "failed"; failure: ApiFailure };

export type ReviewSnapshot = {
  /** The choice that the person set, which the latest review prices. */
  wanted: PlanSelection;
  review: ReviewState;
};

export type ReviewStore = {
  readonly state: Store<ReviewSnapshot>;
  /** Review the first choice once for this page. */
  ensure(): Promise<void>;
  /** Set new add-on quantities and review them. */
  edit(addOns: PlanAddOns): Promise<void>;
  /** Review the wanted choice again, such as after a failed review. */
  retry(): Promise<void>;
};

/** The review of exactly the wanted choice, or null while none is shown. */
export function reviewedChoice(
  snapshot: ReviewSnapshot,
): CheckoutReviewPage | null {
  const { review, wanted } = snapshot;
  if (review.status !== "ready") return null;
  return sameSelection(review.page.choice.selection, wanted)
    ? review.page
    : null;
}

export function createReviewStore(options: {
  initial: PlanSelection;
  read: (wanted: PlanSelection) => Promise<ApiResult<CheckoutReviewPage>>;
  report: <T>(result: ApiResult<T>) => ApiResult<T>;
}): ReviewStore {
  const { read, report } = options;
  const state = createStore<ReviewSnapshot>({
    wanted: options.initial,
    review: { status: "loading" },
  });
  let latest = 0;
  let first: Promise<void> | null = null;

  const review = async (wanted: PlanSelection): Promise<void> => {
    latest += 1;
    const mine = latest;
    state.set({ wanted, review: { status: "loading" } });
    const result = report(await read(wanted));
    if (mine !== latest) return;
    state.set({
      wanted,
      review: result.ok
        ? { status: "ready", page: result.value }
        : { status: "failed", failure: result.failure },
    });
  };

  return {
    state,
    ensure() {
      first ??= review(state.get().wanted);
      return first;
    },
    edit(addOns) {
      const { tier } = state.get().wanted;
      return review({ tier, add_ons: { ...addOns } });
    },
    retry() {
      return review(state.get().wanted);
    },
  };
}
