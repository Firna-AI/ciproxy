import { defineScreen } from "@mokly/mokly";

import { planDoc } from "../../../../components/state-schema.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { PlanUsageScreen } from "../../../../screens/billing/plan-usage.tsx";

export const loading = defineScreen({
  slug: "loading",
  title: "While plan and usage load",
  description: "Skeleton panels keep the layout of the page.",
  relatedDocs: [planDoc],
  address: orgAddress("billing"),
  render: (_props, context) => (
    <PlanUsageScreen viewport={context.viewport} state="loading" />
  ),
});

export const error = defineScreen({
  slug: "error",
  title: "When plan and usage fail to load",
  description: "The shared failed-read state with Try again.",
  relatedDocs: [planDoc],
  address: orgAddress("billing"),
  render: (_props, context) => (
    <PlanUsageScreen viewport={context.viewport} state="error" />
  ),
});
