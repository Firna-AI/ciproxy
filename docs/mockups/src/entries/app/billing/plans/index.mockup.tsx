import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { related } from "../../../../components/related-docs.ts";
import { stateSchema } from "../../../../components/state-schema.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { PlansScreen } from "../../../../screens/billing/plans.tsx";

export const plans = defineScreen({
  title: "Plans",
  description:
    "The three tiers side by side for mokly-ai on Free. Team covers the use of this month and carries the badge; Choose Team opens the checkout review. Each tier lists what it adds, and a feature that ships after the launch says Coming later.",
  rationale:
    "Each tier keeps its six features, so the tiers stay as priced, and no page sells a feature as ready before it ships.",
  relatedDocs: related("billing", "metering", "security"),
  address: orgAddress("billing/plans"),
  useCasePaths: [paths.flowFreeToTeam],
  propSchema: stateSchema(["from-free", "on-team", "inactive-organization"]),
  props: { state: "from-free" },
  render: (props, context) => (
    <PlansScreen viewport={context.viewport} state={props.state} />
  ),
  variants: [
    {
      slug: "on-team",
      title: "Plans, on Team",
      description:
        "The same page for an organization on Team, which sees Business as the next tier. Choose Business reviews a move between paid tiers, and Move to Free asks first.",
      props: { state: "on-team" },
    },
    {
      slug: "inactive-organization",
      title: "Plans, with an inactive organization",
      description:
        "mokly-ai on Free on 8 December. It holds mokly-ai, mokly-labs, and mokly-studio, which is inactive, and three own machines, and Free includes one organization. Team carries the Covers your use badge and clears the plan hold. The breadcrumb opens Plan and usage of the same day. The catalogue has no checkout of December, so Choose Team opens no screen here, and the flows say that the payment happens on Stripe.",
      rationale:
        "The answer D to item 83: the move to Team turns on an organization that waits only for the plan. Every linked organization counts, also an inactive one.",
      props: { state: "inactive-organization" },
      useCasePaths: [
        paths.flowFreeAddsOrganization,
        paths.flowTurnOnOrganization,
      ],
    },
  ],
});
