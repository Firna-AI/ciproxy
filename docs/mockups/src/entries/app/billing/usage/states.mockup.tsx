import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { planDoc } from "../../../../components/state-schema.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { PlanUsageScreen } from "../../../../screens/billing/plan-usage.tsx";

const address = orgAddress("billing");

export const nearLimit = defineScreen({
  slug: "near-limit",
  title: "Starts near the limit",
  description:
    "At the current pace, mokly-ai passes the 25,000 starts of Team before the period ends. The notice offers 10,000 more starts.",
  relatedDocs: [planDoc],
  address,
  render: (_props, context) => (
    <PlanUsageScreen viewport={context.viewport} state="near-limit" />
  ),
});

export const overStarts = defineScreen({
  slug: "over-starts",
  title: "Starts past the limit",
  description:
    "mokly-ai passed 25,000 starts. Jobs keep running; the banner shows on every page, and the add-on is ready to save.",
  relatedDocs: [planDoc],
  address,
  render: (_props, context) => (
    <PlanUsageScreen viewport={context.viewport} state="over-starts" />
  ),
});

export const overMachines = defineScreen({
  slug: "over-machines",
  title: "Own machines past the limit",
  description:
    "A fourth own machine on Team, which includes three. The banner shows on every page, and the extra-machine add-on is ready to save.",
  relatedDocs: [planDoc],
  address,
  render: (_props, context) => (
    <PlanUsageScreen viewport={context.viewport} state="over-machines" />
  ),
});

export const free = defineScreen({
  slug: "free",
  title: "On Free",
  description:
    "mokly-ai on Free on 2 September, with 1,240 of 2,000 starts and 1 own machine. Free has no add-ons and no Stripe customer.",
  relatedDocs: [planDoc],
  address,
  render: (_props, context) => (
    <PlanUsageScreen viewport={context.viewport} state="free" />
  ),
});

export const freeOver = defineScreen({
  slug: "free-over",
  title: "Free starts passed",
  description:
    "On 4 September mokly-ai passed the 2,000 starts of Free. Jobs keep running, and the banner offers Team.",
  relatedDocs: [planDoc],
  address,
  useCasePaths: [paths.flowFreeToTeam],
  render: (_props, context) => (
    <PlanUsageScreen viewport={context.viewport} state="free-over" />
  ),
});
