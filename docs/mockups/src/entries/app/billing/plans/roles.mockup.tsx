import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { PlansScreen } from "../../../../screens/billing/plans.tsx";

const address = orgAddress("billing/plans");

export const viewer = defineScreen({
  slug: "viewer",
  title: "Plans, seen by a Viewer",
  description:
    "Jonas Weber, a Viewer, compares the tiers of mokly-ai on Team. In place of Choose Business and Move to Free, he reads Ask an Admin or a Billing person. Compare plans on Plan and usage opens this page for him.",
  rationale:
    "A Viewer sees Plan and usage read-only, so the plans show what each tier gives without an action that changes the payment.",
  relatedDocs: related("billing", "security"),
  address,
  render: (_props, context) => (
    <PlansScreen viewport={context.viewport} state="on-team" role="viewer" />
  ),
});

export const billing = defineScreen({
  slug: "billing",
  title: "Plans, seen by a Billing person",
  description:
    "Dana Kim has the Billing role, so she chooses a plan as an Admin does: Choose Business reviews a move between paid tiers, and Move to Free asks first. Settings is not in her navigation.",
  relatedDocs: related("billing", "security"),
  address,
  render: (_props, context) => (
    <PlansScreen viewport={context.viewport} state="on-team" role="billing" />
  ),
});
