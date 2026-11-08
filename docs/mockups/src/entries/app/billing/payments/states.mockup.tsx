import { defineScreen } from "@mokly/mokly";

import { related, relatedMvp } from "../../../../components/related-docs.ts";
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

export const paymentFailedOverLimit = defineScreen({
  slug: "payment-failed-over-limit",
  title: "A failed payment above a soft limit",
  description:
    "Stripe could not charge the card for the December period, and mokly-ai runs 4 own machines on Team, which includes 3. The page shows one banner, the failed payment. Own machines still shows 1 over, and the add-ons suggest nothing while the card fails.",
  rationale:
    "Design call 42: at a failed payment, the payment banner replaces the soft-limit banner, so a page keeps one banner.",
  relatedDocs: relatedMvp("billing", "metering"),
  address,
  render: (_props, context) => (
    <PlanUsageScreen
      viewport={context.viewport}
      state="payment-failed-over-limit"
    />
  ),
});

export const movedToFree = defineScreen({
  slug: "moved-to-free",
  title: "Moved to Free after failed payments",
  description:
    "Stripe ended the subscription after its retries on 18 December, so mokly-ai is on Free from that day, and the Free period counts only the starts since then. Its 3 own machines pass the Free limit; jobs keep running and the banner offers a plan. At the pace of that first day, 420 starts, the starts pass the 2,000 of Free on 22 December, so the pace notice and its offer show below the banner.",
  rationale:
    "The use decides the pace notice of Free, as metering.md says: the starts are below the included starts, and the expected starts pass them. The banner of the ended tier stays the one banner of the page.",
  relatedDocs: related("billing", "security"),
  address,
  render: (_props, context) => (
    <PlanUsageScreen viewport={context.viewport} state="moved-to-free" />
  ),
});
