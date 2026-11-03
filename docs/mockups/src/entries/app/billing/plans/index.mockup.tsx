import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { planDoc, stateSchema } from "../../../../components/state-schema.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { PlansScreen } from "../../../../screens/billing/plans.tsx";

export const plans = defineScreen({
  title: "Plans",
  description:
    "The three tiers side by side for mokly-ai on Free. Team covers the use of this month and carries the badge; Choose Team opens the checkout review.",
  relatedDocs: [planDoc],
  address: orgAddress("billing/plans"),
  useCasePaths: [paths.flowFreeToTeam],
  propSchema: stateSchema(["from-free", "on-team"]),
  props: { state: "from-free" },
  render: (props, context) => (
    <PlansScreen viewport={context.viewport} state={props.state} />
  ),
  variants: [
    {
      slug: "on-team",
      title: "Plans, on Team",
      description:
        "The same page for an organization on Team, which sees Business as the next tier.",
      props: { state: "on-team" },
    },
  ],
});
