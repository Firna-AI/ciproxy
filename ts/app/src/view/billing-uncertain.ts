// A billing request whose failure proves no result: no answer, a server
// error, an unreadable success, or a plain conflict. A change can take effect
// while its answer fails, and the server tells an unknown Stripe result and a
// definite refusal apart only by its own words. So the page then says only
// what a fresh Plan and usage read proves, and never that the request charged
// or did not charge. A refusal that the server names, such as a 403, keeps
// its own words.

import { failureStatus, type ApiFailure } from "../api/client.ts";
import type { NoticeTone } from "./page-model.ts";

export function uncertainFailure(failure: ApiFailure): boolean {
  const status = failureStatus(failure);
  if (status === null || status >= 500) return true;
  if (status >= 200 && status < 300) return true;
  if (status !== 409) return false;
  return failure.kind !== "problem" || failure.problem.code === "conflict";
}

/** The words of a payment whose result the page did not get. */
export const unknownPayment = {
  title: "Unable to confirm this payment",
  body: "ciproxy did not get the result. Plan and usage shows the current plan. Jobs keep running.",
} as const;

export type FailureWords = {
  tone: NoticeTone;
  title: string;
  body: string;
  /** The label of the button that asks again. */
  retryLabel: string;
  /**
   * The label of the button that only closes the dialog. After a result that
   * the page did not get, it says Close, not Keep, because Move to Free may
   * have taken effect.
   */
  dismissLabel: string;
};

/** The words of a failed Move to Free, which ends `plan` after `lastDay`. */
export function moveToFreeFailure(
  failure: ApiFailure,
  plan: string,
  lastDay: string,
): FailureWords {
  if (uncertainFailure(failure)) {
    return {
      tone: "warning",
      title: "Unable to confirm Move to Free",
      body: `ciproxy did not get the result. Plan and usage shows whether ${plan} ends on ${lastDay}.`,
      retryLabel: "Try again",
      dismissLabel: "Close",
    };
  }
  return {
    tone: "danger",
    title: "Move to Free did not finish",
    body: "Your plan stays as it was. Try again.",
    retryLabel: "Move to Free",
    dismissLabel: `Keep ${plan}`,
  };
}
