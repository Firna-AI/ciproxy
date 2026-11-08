import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../../components/related-docs.ts";
import { orgAddress } from "../../../../../fixtures/org.ts";
import { CheckoutReviewScreen } from "../../../../../screens/billing/checkout-review.tsx";

const address = orgAddress("billing/checkout");

export const review = defineScreen({
  slug: "review",
  title: "Review Team, as the API prices it",
  description:
    "The review of Team from Free on 4 September, as the API prices it: the plan, the add-on quantities, a monthly total of $49 a month, and $49 charged today, then on the 4th of each month. The API gives no unit prices, so the quantities and the summary show no line amounts. Continue to payment opens Stripe Checkout.",
  relatedDocs: related("billing", "metering"),
  address,
  render: (_props, context) => (
    <CheckoutReviewScreen
      viewport={context.viewport}
      move="to-team"
      api="priced"
    />
  ),
});

export const unpriced = defineScreen({
  slug: "unpriced",
  title: "A review that the API cannot price",
  description:
    "The review of Business from Team on 18 November. The runtime does not price a move between paid tiers yet, so the summary says Unable to price this change, with Try again and Back to plans, and no button pays.",
  rationale:
    "No button pays for a choice that the page does not show with its price.",
  relatedDocs: related("billing", "metering"),
  address,
  render: (_props, context) => (
    <CheckoutReviewScreen
      viewport={context.viewport}
      move="to-business"
      api="unpriced"
    />
  ),
});

const quoteRationale =
  "A move between paid plans pays exactly the charge of its review. The server stores the quote of each review, the payment names it, and the API refuses a quote that is unknown, already used, or out of date. A refused payment never falls back to a new price without a new review.";

export const paidReview = defineScreen({
  slug: "paid-review",
  title: "Review Business, priced by the API",
  description:
    "The review of Business from Team on 18 November, once the API prices a move between paid plans. Stripe charges $133.33 today for the rest of this period, which ends on 3 December, then $299 on the 4th of each month. Move to Business pays exactly this reviewed charge. In the catalogue, Move to Business opens the refusal of a review that is out of date.",
  rationale: quoteRationale,
  relatedDocs: related("billing", "metering"),
  address,
  render: (_props, context) => (
    <CheckoutReviewScreen
      viewport={context.viewport}
      move="to-business"
      api="priced"
    />
  ),
});

export const quoteRefused = defineScreen({
  slug: "quote-refused",
  title: "A review that is out of date at payment",
  description:
    "Move to Business named the quote of its review, and the API refused it, because the quote was unknown, already used, or out of date. The payment did not start. The summary shows no price and asks for a new review. Review again prices the change again.",
  rationale: quoteRationale,
  relatedDocs: related("billing", "metering"),
  address,
  render: (_props, context) => (
    <CheckoutReviewScreen
      viewport={context.viewport}
      move="to-business"
      api="refused"
    />
  ),
});
