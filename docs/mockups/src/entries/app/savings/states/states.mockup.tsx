import { defineScreen } from "@mokly/mokly";

import { planDoc } from "../../../../components/state-schema.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { SavingsScreen } from "../../../../screens/savings/savings.tsx";

const address = orgAddress("savings");

export const missingCosts = defineScreen({
  slug: "missing-costs",
  title: "Machine costs are missing",
  description:
    "No machine has a monthly cost yet, so the report cannot show savings. It still shows the alternatives and the Boat and plan costs.",
  relatedDocs: [planDoc],
  address,
  render: (_props, context) => (
    <SavingsScreen viewport={context.viewport} state="missing-costs" />
  ),
});

export const empty = defineScreen({
  slug: "empty",
  title: "No jobs yet",
  description: "A new organization before its first job.",
  relatedDocs: [planDoc],
  address,
  render: (_props, context) => (
    <SavingsScreen viewport={context.viewport} state="empty" />
  ),
});

export const loading = defineScreen({
  slug: "loading",
  title: "While the report loads",
  description: "Skeleton bars keep the layout of the report.",
  relatedDocs: [planDoc],
  address,
  render: (_props, context) => (
    <SavingsScreen viewport={context.viewport} state="loading" />
  ),
});

export const error = defineScreen({
  slug: "error",
  title: "When the report fails to load",
  description: "The shared failed-read state with Try again.",
  relatedDocs: [planDoc],
  address,
  render: (_props, context) => (
    <SavingsScreen viewport={context.viewport} state="error" />
  ),
});
