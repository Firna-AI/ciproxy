import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { SavingsScreen } from "../../../../screens/savings/savings.tsx";

const address = orgAddress("savings");

export const missingCosts = defineScreen({
  slug: "missing-costs",
  title: "Machine costs are missing",
  description:
    "No machine has a monthly cost yet, so the report cannot show savings. It still shows the alternatives and the Boat and plan costs.",
  relatedDocs: related("metering"),
  address,
  render: (_props, context) => (
    <SavingsScreen viewport={context.viewport} state="missing-costs" />
  ),
});

export const empty = defineScreen({
  slug: "empty",
  title: "No jobs yet",
  description:
    "A new organization before its first job. The range control stays above the empty report.",
  relatedDocs: related("metering"),
  address,
  render: (_props, context) => (
    <SavingsScreen viewport={context.viewport} state="empty" />
  ),
});

export const emptyRange = defineScreen({
  slug: "empty-range",
  title: "A range with no jobs",
  description:
    "A what-if: a workspace whose first job ran in November opens Last month. The range control stays, so the person can choose another range.",
  relatedDocs: related("metering"),
  address,
  render: (_props, context) => (
    <SavingsScreen viewport={context.viewport} state="empty-range" />
  ),
});

export const loading = defineScreen({
  slug: "loading",
  title: "While the report loads",
  description: "Skeleton bars keep the layout of the report.",
  relatedDocs: related("metering"),
  address,
  render: (_props, context) => (
    <SavingsScreen viewport={context.viewport} state="loading" />
  ),
});

export const error = defineScreen({
  slug: "error",
  title: "When the report fails to load",
  description: "The shared failed-read state with Try again.",
  relatedDocs: related("metering"),
  address,
  render: (_props, context) => (
    <SavingsScreen viewport={context.viewport} state="error" />
  ),
});
