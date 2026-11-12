// A payment from one page: the reviewed plan or the quoted add-ons. Only one
// submission runs at a time, from its request until the workspace has been
// read again after its outcome, so no second payment can start from the
// state before the first one. A new submission also waits until the server
// confirms no other payment. Retry sends the same submission again, which
// keeps the original quote of a paid change and its payment. Only the
// server's outcome changes the page: an open payment leaves for Stripe,
// and every other outcome reads the workspace again.

import type { ApiFailure, ApiResult } from "../api/client.ts";
import type { PaymentOutcome } from "../api/billing-types.ts";
import {
  isReviewRequired,
  type PaymentSubmission,
} from "../api/billing-payment.ts";
import { createStore, type Store } from "./store.ts";

export type PaymentSnapshot =
  | { status: "idle" }
  | { status: "submitting"; submission: PaymentSubmission }
  | {
      status: "settled";
      submission: PaymentSubmission;
      outcome: PaymentOutcome;
      /** The workspace is read again after the outcome; nothing pays yet. */
      reading: boolean;
    }
  | {
      status: "failed";
      submission: PaymentSubmission;
      failure: ApiFailure;
      reading: boolean;
    };

/** Whether a payment of the page runs or waits for the read after it. */
export function paymentBusy(snapshot: PaymentSnapshot): boolean {
  if (snapshot.status === "submitting") return true;
  return snapshot.status !== "idle" && snapshot.reading;
}

export type PaymentStore = {
  readonly state: Store<PaymentSnapshot>;
  /**
   * Pay for a new submission. Nothing happens while a submission runs, or
   * while `canStart` says that the server still waits for a payment.
   */
  submit(submission: PaymentSubmission): Promise<void>;
  /** Send the last submission again, to continue or check its payment. */
  retry(): Promise<void>;
  /** Forget the last outcome, so the page can review a new choice. */
  reset(): void;
};

export function createPaymentStore(options: {
  send: (submission: PaymentSubmission) => Promise<ApiResult<PaymentOutcome>>;
  mutate: <T>(run: () => Promise<ApiResult<T>>) => Promise<ApiResult<T>>;
  /** Whether a new payment may start now, from the latest server state. */
  canStart: () => boolean;
  /** Open a Stripe page in place of the dashboard. */
  leave: (url: string) => void;
  /** Read the changed workspace again: its pages and its banner. */
  settled: () => Promise<void>;
}): PaymentStore {
  const { send, mutate, canStart, leave, settled } = options;
  const state = createStore<PaymentSnapshot>({ status: "idle" });
  /**
   * Set from the request until the read after its outcome ends. An open
   * payment keeps it, because the page leaves for Stripe.
   */
  let busy = false;

  const run = async (submission: PaymentSubmission): Promise<void> => {
    busy = true;
    state.set({ status: "submitting", submission });
    const result = await mutate(() => send(submission));
    if (result.ok && result.value.state === "open") {
      state.set({
        status: "settled",
        submission,
        outcome: result.value,
        reading: true,
      });
      leave(result.value.url);
      return;
    }
    const after: PaymentSnapshot = result.ok
      ? { status: "settled", submission, outcome: result.value, reading: true }
      : {
          status: "failed",
          submission,
          failure: result.failure,
          reading: true,
        };
    // Recovery from a refused quote needs the billing facts after the refusal.
    const reviewRefused =
      submission.kind === "plan" &&
      !result.ok &&
      isReviewRequired(result.failure);
    if (!reviewRefused) state.set(after);
    try {
      await settled();
    } finally {
      busy = false;
      state.set({ ...after, reading: false });
    }
  };

  return {
    state,
    async submit(submission) {
      if (busy || !canStart()) return;
      await run(submission);
    },
    async retry() {
      const current = state.get();
      if (busy) return;
      if (current.status !== "settled" && current.status !== "failed") return;
      await run(current.submission);
    },
    reset() {
      if (!busy) state.set({ status: "idle" });
    },
  };
}
