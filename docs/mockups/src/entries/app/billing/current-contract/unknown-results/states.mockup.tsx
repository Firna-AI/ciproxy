import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../../components/paths.ts";
import { related } from "../../../../../components/related-docs.ts";
import { orgAddress } from "../../../../../fixtures/org.ts";
import { CheckoutReviewScreen } from "../../../../../screens/billing/checkout-review.tsx";
import { PlanUsageScreen } from "../../../../../screens/billing/plan-usage.tsx";
import { PlansScreen } from "../../../../../screens/billing/plans.tsx";

const rationale =
  "No answer, a server error, an unreadable success, and a plain conflict prove no result, and a change can take effect while its answer fails. So only the Plan and usage read after it decides the words: the choice is the current plan, a payment or a billing change waits, or the result stays unknown. The page never says that such a payment charged or did not charge, and nothing repeats without a click.";

const checkout = orgAddress("billing/checkout");

export const paymentUnknown = defineScreen({
  slug: "payment-unknown",
  title: "A payment result that the page did not get",
  description:
    "Move to Business got no readable answer. The Plan and usage read after it shows neither Business nor a payment that waits, so the notice says only that the page could not confirm the payment, and the pay button stays off. Try again sends the same payment again, or reads Plan and usage again when that read failed.",
  rationale,
  relatedDocs: related("billing", "security"),
  address: checkout,
  render: (_props, context) => (
    <CheckoutReviewScreen
      viewport={context.viewport}
      move="to-business"
      api="priced"
      outcome="unknown"
    />
  ),
});

export const paymentCurrent = defineScreen({
  slug: "payment-current",
  title: "The read shows the choice as the current plan",
  description:
    "After the same answer, the Plan and usage read shows Business as the current plan. The notice says only that Business is the current plan. It does not say that this payment made the change.",
  rationale,
  relatedDocs: related("billing", "security"),
  address: checkout,
  render: (_props, context) => (
    <CheckoutReviewScreen
      viewport={context.viewport}
      move="to-business"
      api="priced"
      outcome="current"
    />
  ),
});

export const paymentWaits = defineScreen({
  slug: "payment-waits",
  title: "The read shows a payment that waits",
  description:
    "After the same answer, the Plan and usage read shows a payment or a billing change in progress. The notice says so, and Open plan and usage shows it.",
  rationale,
  relatedDocs: related("billing", "security"),
  address: checkout,
  render: (_props, context) => (
    <CheckoutReviewScreen
      viewport={context.viewport}
      move="to-business"
      api="priced"
      outcome="waits"
    />
  ),
});

export const addOnsUnknown = defineScreen({
  slug: "add-ons-unknown",
  title: "Save add-ons with no result",
  description:
    "Save add-ons sent the offer, 1 block of starts, and got no readable answer. The Plan and usage read after it shows the saved add-ons as before and no payment that waits, so the notice says that the page could not confirm the payment.",
  rationale,
  relatedDocs: related("billing", "security"),
  address: orgAddress("billing"),
  render: (_props, context) => (
    <PlanUsageScreen
      viewport={context.viewport}
      state="over-starts"
      api={{
        plans: paths.plansOnTeam,
        edited: { starts: 1, machines: 0 },
        outcome: "unknown",
      }}
    />
  ),
});

export const moveToFreeUnknown = defineScreen({
  slug: "move-to-free-unknown",
  title: "Move to Free with no result",
  description:
    "Move to Free got no readable answer. The dialog says that the page could not confirm it, and that Plan and usage shows whether Team ends on 3 December. Try again asks again. Close only closes the dialog, because Move to Free may have taken effect.",
  rationale,
  relatedDocs: related("billing", "security"),
  address: orgAddress("billing/plans"),
  render: (_props, context) => (
    <PlansScreen viewport={context.viewport} state="move-to-free-unknown" />
  ),
});
