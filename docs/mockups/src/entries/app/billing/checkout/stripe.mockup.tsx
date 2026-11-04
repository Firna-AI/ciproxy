import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { related } from "../../../../components/related-docs.ts";
import { StripeCheckoutScreen } from "../../../../screens/billing/stripe-checkout.tsx";

export const stripe = defineScreen({
  title: "Stripe Checkout",
  description:
    "The payment page that Stripe hosts. ciproxy supplies the line items and the return pages; the card never reaches ciproxy.",
  rationale:
    "Shown so that the flow is complete. Stripe owns the layout of this page; ciproxy sets its branding, the Graphite colours and a black button, and the line items.",
  relatedDocs: related("billing", "security"),
  address: "checkout.stripe.com/c/pay/cs_live_b1Qx7",
  useCasePaths: [paths.flowFreeToTeam],
  render: (_props, context) => (
    <StripeCheckoutScreen viewport={context.viewport} />
  ),
});
