import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../../components/related-docs.ts";
import { host, orgAddress } from "../../../../../fixtures/org.ts";
import { CheckoutResultScreen } from "../../../../../screens/billing/checkout-result.tsx";

const address = orgAddress("billing/checkout/return");

export const confirmed = defineScreen({
  slug: "confirmed",
  title: "The plan is confirmed, as the API gives it",
  description:
    "mokly-ai is on Team, and its first Team period starts today. The API names no receipt address, so the page says only that Stripe emails the receipt.",
  relatedDocs: related("billing", "metering"),
  address,
  render: (_props, context) => (
    <CheckoutResultScreen viewport={context.viewport} state="api-confirmed" />
  ),
});

export const open = defineScreen({
  slug: "open",
  title: "The Checkout is still open",
  description:
    "The return while the Checkout of this payment is still open on Stripe. Stripe took no payment yet. Continue payment opens the same Checkout, and Back to plans opens the plans.",
  relatedDocs: related("billing", "security"),
  address,
  render: (_props, context) => (
    <CheckoutResultScreen viewport={context.viewport} state="open" />
  ),
});

export const unfinished = defineScreen({
  slug: "unfinished",
  title: "The payment did not finish, as the API gives it",
  description:
    "Stripe took no payment, and mokly-ai is still on Free. Try again opens the plans, because a new payment starts from a new review. Open plan and usage shows the use of the day.",
  relatedDocs: related("billing", "metering"),
  address,
  render: (_props, context) => (
    <CheckoutResultScreen viewport={context.viewport} state="unfinished" />
  ),
});

export const noWorkspace = defineScreen({
  slug: "no-workspace",
  title: "A return that names no workspace",
  description:
    "Stripe returns to one address for every workspace. This tab saved no workspace before it left for Stripe, for example because the payment started in another browser. The page says so in place of a guess. Stripe also tells ciproxy about the payment, so the plan changes once Stripe confirms it.",
  rationale:
    "The page never guesses the workspace of a payment from the last workspace that the person opened.",
  relatedDocs: related("billing", "security"),
  address: `${host}/billing/checkout/return`,
  render: (_props, context) => (
    <CheckoutResultScreen viewport={context.viewport} state="no-workspace" />
  ),
});
