// What a page shows after a payment, from the server's outcome: a plan that
// Stripe confirmed, a payment that Stripe still confirms, a payment to finish
// on its Stripe page, a card that failed, or a refusal. When the page did not
// get the result, it says only what the Plan and usage read after it proves:
// the submitted choice is the current plan, or a payment or a billing change
// waits. Otherwise the result stays unknown. Retry sends the same submission
// again, with the same quote of a paid change, so the server continues the
// same payment. A refused quote of a plan shows no notice: the Checkout
// summary asks for a new review, and a retry would send that quote again.

import type { PlanUsagePage } from "../api/billing-types.ts";
import { failureStatus } from "../api/client.ts";
import {
  isReviewRequired,
  sameSelection,
  submittedSelection,
  type PaymentSubmission,
} from "../api/billing-payment.ts";
import { statusWaits, type PageSnapshot } from "../state/billing-page.ts";
import type { PaymentSnapshot } from "../state/billing-payment.ts";
import { askPayer, selectionText } from "./billing-format.ts";
import { billingHref, plansHref } from "./billing-routes.ts";
import { uncertainFailure, unknownPayment } from "./billing-uncertain.ts";
import type { NoticeTone } from "./page-model.ts";

export type OutcomeAction =
  | { kind: "link"; label: string; href: string; newTab: boolean }
  | { kind: "retry"; label: string }
  /** Read Plan and usage again, after that read failed. */
  | { kind: "reload"; label: string }
  | { kind: "portal"; label: string };

export type OutcomeModel = {
  /** A plan that the server confirmed shows as done; the rest as notices. */
  done: boolean;
  tone: NoticeTone;
  title: string;
  body: string;
  actions: readonly OutcomeAction[];
};

const link = (label: string, href: string, newTab = false): OutcomeAction => ({
  kind: "link",
  label,
  href,
  newTab,
});

export function outcomeModel(
  snapshot: PaymentSnapshot,
  workspace: { id: string; name: string },
  /** The latest Plan and usage snapshot of the workspace. */
  current: PageSnapshot<PlanUsagePage>,
): OutcomeModel | null {
  if (snapshot.status === "idle" || snapshot.status === "submitting") {
    return null;
  }
  const usage = link("Open plan and usage", billingHref(workspace.id));
  if (snapshot.status === "failed") {
    if (
      snapshot.submission.kind === "plan" &&
      isReviewRequired(snapshot.failure)
    ) {
      return null;
    }
    const status = failureStatus(snapshot.failure);
    if (uncertainFailure(snapshot.failure)) {
      return uncertainModel(snapshot, current, usage);
    }
    if (status === 403) {
      return {
        done: false,
        tone: "danger",
        title: "Only an Admin or a Billing person can pay",
        body: askPayer,
        actions: [],
      };
    }
    return {
      done: false,
      tone: "danger",
      title: "The payment did not start",
      body: "Your plan stays as it was, and jobs keep running. Try again.",
      actions: [{ kind: "retry", label: "Try again" }],
    };
  }
  const { outcome, submission } = snapshot;
  const choice = selectionText(submittedSelection(submission));
  switch (outcome.state) {
    case "open":
      return {
        done: false,
        tone: "info",
        title: "Opening Stripe",
        body: "Stripe Checkout opens in this tab.",
        actions: [link("Continue to payment", outcome.url)],
      };
    case "confirming":
      return {
        done: false,
        tone: "info",
        title: "Stripe is confirming your payment",
        body: "Your plan changes when Stripe confirms it. Plan and usage updates by itself, and jobs keep running.",
        actions: [usage],
      };
    case "confirmed":
      return {
        done: true,
        tone: "info",
        title:
          submission.kind === "add-ons"
            ? "Your add-ons are saved"
            : `${workspace.name} is on ${choice}`,
        body:
          submission.kind === "add-ons"
            ? "Stripe charges the difference for the rest of this period."
            : "Stripe confirmed the payment and emails the receipt.",
        actions: [usage],
      };
    case "unfinished":
      return {
        done: false,
        tone: "warning",
        title: "Your plan did not change",
        body: "Stripe took no payment, and jobs keep running.",
        actions: [
          { kind: "retry", label: "Try again" },
          link("Back to plans", plansHref(workspace.id)),
        ],
      };
    case "payment_required":
      return outcome.payment_url
        ? {
            done: false,
            tone: "warning",
            title: "Your payment is not finished",
            body: "Finish it on its Stripe page. Your plan changes when Stripe confirms the payment.",
            actions: [
              link("Open the Stripe page", outcome.payment_url, true),
              { kind: "retry", label: "Check again" },
            ],
          }
        : {
            done: false,
            tone: "danger",
            title: "Stripe could not charge your card",
            body: "Your plan stays as it was, and jobs keep running. Update the card, then try again.",
            actions: [
              { kind: "portal", label: "Update card" },
              { kind: "retry", label: "Try again" },
            ],
          };
  }
}

/**
 * A failed payment that proves no result. While the read after it runs, or
 * when that read failed, the result is unknown. A fresh read proves either
 * the submitted choice as the current plan, or a payment or a billing change
 * that waits. Anything else leaves the result unknown.
 */
function uncertainModel(
  snapshot: { submission: PaymentSubmission; reading: boolean },
  current: PageSnapshot<PlanUsagePage>,
  usage: OutcomeAction,
): OutcomeModel {
  const unknown = (again: OutcomeAction): OutcomeModel => ({
    done: false,
    tone: "warning",
    ...unknownPayment,
    actions: [usage, again],
  });
  const retry: OutcomeAction = { kind: "retry", label: "Try again" };
  if (snapshot.reading) return unknown(retry);
  if (current.status !== "ready" || current.refreshFailure !== null) {
    return unknown({ kind: "reload", label: "Try again" });
  }
  const { status } = current.page;
  if (
    sameSelection(status.selection, submittedSelection(snapshot.submission))
  ) {
    return {
      done: false,
      tone: "info",
      title: `${selectionText(status.selection)} is your current plan`,
      body: "Plan and usage shows it.",
      actions: [usage],
    };
  }
  if (statusWaits(status)) {
    return {
      done: false,
      tone: "warning",
      title: "A payment or a billing change is in progress",
      body: "Plan and usage shows it until it ends. Jobs keep running.",
      actions: [usage],
    };
  }
  return unknown(retry);
}
