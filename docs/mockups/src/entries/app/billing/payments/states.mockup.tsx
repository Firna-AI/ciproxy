import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { PlanUsageScreen } from "../../../../screens/billing/plan-usage.tsx";

const address = orgAddress("billing");

export const paymentFailed = defineScreen({
  slug: "payment-failed",
  title: "A renewal payment failed",
  description:
    "Stripe could not charge the card for the December period. Team stays active while Stripe retries; Update card opens the Stripe billing portal.",
  relatedDocs: related("billing", "security"),
  address,
  render: (_props, context) => (
    <PlanUsageScreen viewport={context.viewport} state="payment-failed" />
  ),
});

export const paymentFailedViewer = defineScreen({
  slug: "payment-failed-viewer",
  title: "A failed payment, seen by a Viewer",
  description:
    "Jonas Weber, a Viewer, sees the same banner and the same page, read-only. In place of Update card, Change plan, and Invoices and card, he reads Ask an Admin or a Billing person. Compare plans opens the plans without their buttons, and Settings is not in his navigation.",
  rationale:
    "Everyone sees the state of the organization, and only an Admin or a Billing person acts on it.",
  relatedDocs: related("billing", "security"),
  address,
  render: (_props, context) => (
    <PlanUsageScreen
      viewport={context.viewport}
      state="payment-failed"
      role="viewer"
    />
  ),
});

export const movedToFree = defineScreen({
  slug: "moved-to-free",
  title: "Moved to Free after failed payments",
  description:
    "Stripe ended the subscription after its retries on 18 December, so mokly-ai is on Free from that day, and the Free period counts only the starts since then. Its 3 own machines pass the Free limit; jobs keep running and the banner offers a plan.",
  relatedDocs: related("billing", "security"),
  address,
  render: (_props, context) => (
    <PlanUsageScreen viewport={context.viewport} state="moved-to-free" />
  ),
});
