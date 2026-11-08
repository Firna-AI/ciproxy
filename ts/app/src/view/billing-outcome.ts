// What a page shows after a payment, from the server's outcome only: a plan
// that Stripe confirmed, a payment that Stripe still confirms, a bank that
// asks to confirm, a card that failed, or a request that did not reach the
// server. Retry sends the same reviewed choice again, which continues the
// same payment on the server.

import { failureStatus } from "../api/client.ts";
import { submittedSelection } from "../api/billing-payment.ts";
import type { PaymentSnapshot } from "../state/billing-payment.ts";
import { askPayer, selectionText } from "./billing-format.ts";
import { billingHref, plansHref } from "./billing-routes.ts";
import type { NoticeTone } from "./page-model.ts";

export type OutcomeAction =
  | { kind: "link"; label: string; href: string; newTab: boolean }
  | { kind: "retry"; label: string }
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
): OutcomeModel | null {
  if (snapshot.status === "idle" || snapshot.status === "submitting") {
    return null;
  }
  const usage = link("Open plan and usage", billingHref(workspace.id));
  if (snapshot.status === "failed") {
    const status = failureStatus(snapshot.failure);
    if (status === 409) {
      return {
        done: false,
        tone: "warning",
        title: "A payment or a billing change is in progress",
        body: "Plan and usage shows it until it ends. Jobs keep running.",
        actions: [usage],
      };
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
      title:
        status === null
          ? "Unable to reach ciproxy"
          : "The payment did not start",
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
            title: "Your bank needs to confirm this payment",
            body: "Confirm it on the Stripe page. Your plan stays as it was until Stripe confirms the payment.",
            actions: [
              link("Confirm the payment", outcome.payment_url, true),
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
