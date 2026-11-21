import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../../components/paths.ts";
import { related } from "../../../../../components/related-docs.ts";
import { orgAddress } from "../../../../../fixtures/org.ts";
import { PlanUsageScreen } from "../../../../../screens/billing/plan-usage.tsx";
import { PlansScreen } from "../../../../../screens/billing/plans.tsx";

const rationale =
  "The API gives no address of an open payment. This tab keeps the exact Stripe address that the API returned, for its workspace only, and offers it only while a fresh read says that a payment waits. No page starts a second payment.";

export const resume = defineScreen({
  slug: "continue",
  title: "Continue a payment after Checkout was left",
  description:
    "On 4 September the payer chose Team, opened Stripe Checkout, and went back without paying. Stripe keeps that Checkout open, so the API says that a payment waits, and no new payment can start. This tab saved the Stripe page of the payment, so Continue payment opens that same Checkout, also after a reload of the tab.",
  rationale,
  relatedDocs: related("billing", "security"),
  address: orgAddress("billing"),
  render: (_props, context) => (
    <PlanUsageScreen
      viewport={context.viewport}
      state="free-over"
      api={{ plans: paths.plansCurrentContinue, pending: "checkout" }}
    />
  ),
});

export const plansResume = defineScreen({
  slug: "plans-continue",
  title: "Plans while a Checkout is open",
  description:
    "The Plans page of the same moment, which the cancel address of Stripe opens. The notice offers Continue payment, and no plan can be chosen until the payment ends. The cards name no add-on prices, because the API gives none.",
  rationale,
  relatedDocs: related("billing", "security"),
  address: orgAddress("billing/plans"),
  render: (_props, context) => (
    <PlansScreen viewport={context.viewport} state="pending-checkout" />
  ),
});

export const bankResume = defineScreen({
  slug: "bank-continue",
  title: "Continue a payment on its Stripe page after a reload",
  description:
    "In mid-November the payer saved 1 block of starts, and the payment waited on the Stripe page of its unpaid invoice. After a reload, the API still says that a payment waits. This tab saved that Stripe page, so Continue payment opens it in a new tab. The add-ons cannot change until the payment ends.",
  rationale,
  relatedDocs: related("billing", "security"),
  address: orgAddress("billing"),
  render: (_props, context) => (
    <PlanUsageScreen
      viewport={context.viewport}
      state="over-starts"
      api={{ plans: paths.plansOnTeam, pending: "invoice" }}
    />
  ),
});

export const confirming = defineScreen({
  slug: "confirming",
  title: "Stripe confirms a payment",
  description:
    "A payment waits, and this tab saved no Stripe page for it, for example because the payment started in another tab. The notice says that Stripe confirms it, with no action, and the page reads the API again every few seconds until the payment ends.",
  relatedDocs: related("billing", "metering"),
  address: orgAddress("billing"),
  render: (_props, context) => (
    <PlanUsageScreen
      viewport={context.viewport}
      state="free-over"
      api={{ plans: paths.plansCurrentConfirming, pending: "confirming" }}
    />
  ),
});

export const plansConfirming = defineScreen({
  slug: "plans-confirming",
  title: "Plans while Stripe confirms a payment",
  description:
    "The Plans page of the same moment. No plan can be chosen until the payment ends, and the page says so and updates by itself.",
  relatedDocs: related("billing", "metering"),
  address: orgAddress("billing/plans"),
  render: (_props, context) => (
    <PlansScreen viewport={context.viewport} state="pending-confirming" />
  ),
});
