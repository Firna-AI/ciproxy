import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../../components/related-docs.ts";
import { orgAddress } from "../../../../../fixtures/org.ts";
import { CheckoutReviewScreen } from "../../../../../screens/billing/checkout-review.tsx";

const address = orgAddress("billing/checkout");

const rationale =
  "A new payment waits for a fresh, successful Plan and usage read that shows a payer and nothing in progress. The summary, Save add-ons, and the payment itself read one reason for a closed payment, and the summary names it with the action that recovers it.";

export const statusFailed = defineScreen({
  slug: "status-failed",
  title: "Plan and usage did not load",
  description:
    "The review of Business priced the move, but the Plan and usage read failed, so the page cannot tell whether a payment may start. The summary says so in place of the price. Try again reads Plan and usage again, and the review too when it failed. The add-on quantities stay. Nothing pays until the read works and the person clicks Move to Business.",
  rationale: `${rationale} The failed-read words come before the words of a review.`,
  relatedDocs: related("billing", "metering"),
  address,
  render: (_props, context) => (
    <CheckoutReviewScreen
      viewport={context.viewport}
      move="to-business"
      api="status-failed"
    />
  ),
});

export const checking = defineScreen({
  slug: "checking",
  title: "Plan and usage still loads",
  description:
    "The review priced the move while the Plan and usage read still runs. The summary says Checking your plan, and the price and the pay button show when the read ends.",
  rationale,
  relatedDocs: related("billing", "metering"),
  address,
  render: (_props, context) => (
    <CheckoutReviewScreen
      viewport={context.viewport}
      move="to-business"
      api="checking"
    />
  ),
});

export const waiting = defineScreen({
  slug: "waiting",
  title: "A payment or a billing change waits",
  description:
    "The Plan and usage read shows a payment or a billing change in progress, so no second payment starts. The summary says so, and Open plan and usage shows it.",
  rationale,
  relatedDocs: related("billing", "metering"),
  address,
  render: (_props, context) => (
    <CheckoutReviewScreen
      viewport={context.viewport}
      move="to-business"
      api="waiting"
    />
  ),
});

export const notPayer = defineScreen({
  slug: "not-payer",
  title: "A person who cannot pay",
  description:
    "The session or the Plan and usage read says that this person cannot pay, for example after an Admin changed the role. The summary says who can pay, and no button pays.",
  rationale,
  relatedDocs: related("billing", "security"),
  address,
  render: (_props, context) => (
    <CheckoutReviewScreen
      viewport={context.viewport}
      move="to-business"
      api="not-payer"
    />
  ),
});
