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
    "The review of Business from Team on 18 November did not load, for example after no answer or a server error. The summary says Unable to price this change, with Try again and Back to plans, and no button pays. A paid review that comes without its quote shows the same summary.",
  rationale:
    "No button pays for a choice that the page does not show with its price.",
  relatedDocs: related("billing", "paidMoves", "metering"),
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
  "A move between paid plans pays exactly the charge of its review. The server stores the quote of each review, the payment names it, and the API refuses a quote that is unknown, already used, or out of date. A refused payment never falls back to a new price without a new review. A new review sends no payment, so each payment needs its own click.";

export const paidReview = defineScreen({
  slug: "paid-review",
  title: "Review Business, priced by the API",
  description:
    "The review of Business from Team on 18 November, as the API prices it. Stripe charges $133.33 today for the rest of this period, which ends on 3 December, then $299 on the 4th of each month. Move to Business pays exactly this reviewed charge. After Review again, the new review shows here the same way, and it pays only after a new click on Move to Business, also when its charge did not change. In the catalogue, Move to Business opens the refusal of a review that is out of date.",
  rationale: quoteRationale,
  relatedDocs: related("billing", "paidMoves", "metering"),
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
    "Move to Business named the quote of its review, and the API refused it, because the quote was unknown, already used, or out of date. The payment did not start. The page reads Plan and usage again before it shows this summary. When that read fails, or shows a payment or a billing change in progress, the summary of the payment gate shows in its place, with no Review again. When the read shows Business as the current plan, the summary says Business is your current plan, with no Review again. Otherwise the summary shows no price and asks for a new review. Review again only reads a new review and sends no payment. Nothing pays until the person clicks Move to Business on the new review, also when its charge did not change. In the catalogue, Review again opens the paid review.",
  rationale: quoteRationale,
  relatedDocs: related("billing", "paidMoves", "metering"),
  address,
  render: (_props, context) => (
    <CheckoutReviewScreen
      viewport={context.viewport}
      move="to-business"
      api="refused"
    />
  ),
});
