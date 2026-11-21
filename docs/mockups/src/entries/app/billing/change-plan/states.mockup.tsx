import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { related, relatedMvp } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { CheckoutReviewScreen } from "../../../../screens/billing/checkout-review.tsx";
import { PlanUsageScreen } from "../../../../screens/billing/plan-usage.tsx";
import { PlansScreen } from "../../../../screens/billing/plans.tsx";

export const toBusiness = defineScreen({
  slug: "to-business",
  title: "Move between paid tiers",
  description:
    "mokly-ai moves from Team to Business on 18 November. The move takes effect at once and keeps the period, so the 8,640 starts so far count toward Business. Stripe charges the card on file $133.33 for the 16 days left, with no Stripe Checkout.",
  relatedDocs: related("billing", "metering"),
  address: orgAddress("billing/change/business"),
  render: (_props, context) => (
    <CheckoutReviewScreen viewport={context.viewport} move="to-business" />
  ),
});

export const moveToFree = defineScreen({
  slug: "move-to-free",
  title: "Move to Free",
  description:
    "Move to Free on the Plans page asks first. Team and its add-ons stay until the end of the period, then mokly-ai is on Free, and jobs keep running on all three machines.",
  relatedDocs: related("billing", "metering"),
  address: orgAddress("billing/plans"),
  useCasePaths: [paths.flowFreeKeepsOrganizations],
  render: (_props, context) => (
    <PlansScreen viewport={context.viewport} state="move-to-free" />
  ),
});

export const freeScheduled = defineScreen({
  slug: "free-scheduled",
  title: "Team ends at the end of the period",
  description:
    "After Move to Free, Plan and usage shows the day Team ends. Until then, Keep Team opens the Stripe billing portal, which undoes the move.",
  relatedDocs: related("billing", "metering"),
  address: orgAddress("billing"),
  useCasePaths: [paths.flowFreeKeepsOrganizations],
  render: (_props, context) => (
    <PlanUsageScreen viewport={context.viewport} state="free-scheduled" />
  ),
});

export const freeOrganizations = defineScreen({
  slug: "free-organizations",
  title: "On Free with two organizations",
  description:
    "On 4 December, the day after Team ended, mokly-ai is on Free and keeps both its organizations, mokly-ai and mokly-labs. Their jobs keep running on all three machines. Free includes one organization, so the banner of every page says that mokly-ai passed a limit of its plan and offers Team, which covers the use. At the pace of the first day, the starts pass the 2,000 of Free on 9 December, so the page also shows that notice.",
  rationale:
    "A workspace that holds more than one organization when it moves to Free keeps them all, and on Free the number of organizations is a soft limit, with the banner and the offer, as for the machine count. The Organizations tab shows the same banner.",
  relatedDocs: relatedMvp("workspaces", "metering", "billing"),
  address: orgAddress("billing"),
  useCasePaths: [paths.flowFreeKeepsOrganizations],
  render: (_props, context) => (
    <PlanUsageScreen viewport={context.viewport} state="free-organizations" />
  ),
});

export const inactiveOrganization = defineScreen({
  slug: "inactive-organization",
  title: "On Free with an inactive organization",
  description:
    "On 8 December, mokly-ai is still on Free with mokly-ai and mokly-labs, and mokly-studio joins inactive. The banner says that mokly-ai passed a limit of its plan and offers Team. At the pace of the first five days, 1,900 starts, the starts pass the 2,000 of Free on 9 December, so the page shows that notice too. The banner, the notice, and Compare plans open the Plans of that day, and the Plan and usage link of the inactive organization opens this page.",
  rationale:
    "The answer D to item 83: every linked organization counts on Free, also an inactive one. The links of 8 December keep the third organization, so the 4 December page and the September Plans stay as they are.",
  relatedDocs: relatedMvp("workspaces", "metering", "billing"),
  address: orgAddress("billing"),
  render: (_props, context) => (
    <PlanUsageScreen
      viewport={context.viewport}
      state="inactive-organization"
    />
  ),
});
