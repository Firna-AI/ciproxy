import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { CheckoutResultScreen } from "../../../../screens/billing/checkout-result.tsx";

const address = orgAddress("billing/checkout/return");

export const confirming = defineScreen({
  slug: "confirming",
  title: "Confirming the payment",
  description:
    "Stripe returned the customer before ciproxy received the confirmation of the payment. The page waits a few seconds.",
  relatedDocs: related("billing", "metering"),
  address,
  useCasePaths: [paths.flowFreeToTeam],
  render: (_props, context) => (
    <CheckoutResultScreen viewport={context.viewport} state="confirming" />
  ),
});

export const confirmed = defineScreen({
  slug: "confirmed",
  title: "The plan is confirmed",
  description:
    "mokly-ai is on Team. The first Team period starts today, so its starts count from zero against the 25,000 of Team. The Free starts stay in the Free period.",
  relatedDocs: related("billing", "metering"),
  address,
  useCasePaths: [paths.flowFreeToTeam],
  render: (_props, context) => (
    <CheckoutResultScreen viewport={context.viewport} state="confirmed" />
  ),
});

export const cancelled = defineScreen({
  slug: "cancelled",
  title: "The payment did not finish",
  description:
    "The return when the customer leaves Stripe Checkout without paying, for example after the card was declined. The plan did not change.",
  relatedDocs: related("billing", "metering"),
  address,
  render: (_props, context) => (
    <CheckoutResultScreen viewport={context.viewport} state="cancelled" />
  ),
});
