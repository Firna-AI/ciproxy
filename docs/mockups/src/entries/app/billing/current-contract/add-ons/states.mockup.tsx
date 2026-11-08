import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../../components/paths.ts";
import { related } from "../../../../../components/related-docs.ts";
import { orgAddress } from "../../../../../fixtures/org.ts";
import { PlanUsageScreen } from "../../../../../screens/billing/plan-usage.tsx";

const address = orgAddress("billing");
const offerAddOns = { starts: 1, machines: 0 };

export const offer = defineScreen({
  slug: "offer",
  title: "The add-ons that cover the use",
  description:
    "After Use the add-ons that cover your use, the quantities show 1 block of starts, the offer of the API. Its total, $59 a month, is the price that the API sent, and Save add-ons saves exactly that choice. Stripe charges the difference for the rest of this period.",
  relatedDocs: related("billing", "metering"),
  address,
  render: (_props, context) => (
    <PlanUsageScreen
      viewport={context.viewport}
      state="over-starts"
      api={{ plans: paths.plansOnTeam, edited: offerAddOns }}
    />
  ),
});

export const unpriced = defineScreen({
  slug: "unpriced",
  title: "An add-on choice with no price",
  description:
    "The payer set 2 blocks of starts. The API prices only the saved add-ons and the offer, so the total says Monthly total not available, and Save add-ons stays off. Use the add-ons that cover your use sets the offer again.",
  rationale:
    "The page never computes a price. It shows a total only where the API sent one.",
  relatedDocs: related("billing", "metering"),
  address,
  render: (_props, context) => (
    <PlanUsageScreen
      viewport={context.viewport}
      state="over-starts"
      api={{ plans: paths.plansOnTeam, edited: { starts: 2, machines: 0 } }}
    />
  ),
});

export const pace = defineScreen({
  slug: "pace",
  title: "Starts near the limit, with the offer of the API",
  description:
    "At this pace, mokly-ai passes the 25,000 starts of Team on 2 December. The notice names the offer of the API, Team with 1 block of starts, which includes 35,000 starts a month for $59 a month. Review add-ons sets the add-ons to that offer.",
  relatedDocs: related("metering", "billing"),
  address,
  render: (_props, context) => (
    <PlanUsageScreen
      viewport={context.viewport}
      state="near-limit"
      api={{ plans: paths.plansOnTeam }}
    />
  ),
});

export const bank = defineScreen({
  slug: "bank",
  title: "The bank asks to confirm the payment",
  description:
    "Save add-ons sent the offer, and Stripe answered that the bank must confirm the payment. Confirm the payment opens the Stripe page in a new tab, and Check again asks ciproxy again. Save add-ons stays off while the payment waits, and the page shows no second notice of it.",
  relatedDocs: related("billing", "security"),
  address,
  render: (_props, context) => (
    <PlanUsageScreen
      viewport={context.viewport}
      state="over-starts"
      api={{ plans: paths.plansOnTeam, edited: offerAddOns, outcome: "bank" }}
    />
  ),
});

export const card = defineScreen({
  slug: "card",
  title: "The card fails",
  description:
    "Stripe could not charge the card for the offer. Update card opens the Stripe billing portal, and Try again sends the same choice again, which continues the same payment. The plan stays as it was, and jobs keep running.",
  relatedDocs: related("billing", "security"),
  address,
  render: (_props, context) => (
    <PlanUsageScreen
      viewport={context.viewport}
      state="over-starts"
      api={{ plans: paths.plansOnTeam, edited: offerAddOns, outcome: "card" }}
    />
  ),
});
