import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { SavingsScreen } from "../../../../screens/savings/savings.tsx";

const address = orgAddress("savings");

export const negative = defineScreen({
  slug: "negative",
  title: "ciproxy costs more than an alternative",
  description:
    "This month on Sunday 1 November: a quiet day, and the machines cost the same as on a busy day. ciproxy costs more than Blacksmith and less than GitHub-hosted runners. The figure keeps its size and says more than, so the sign of the saving shows in words.",
  rationale:
    "metering.md: when the cost with ciproxy is higher than an alternative, the report shows how much higher it is. The report leads with the cheaper alternative.",
  relatedDocs: related("metering"),
  address,
  render: (_props, context) => (
    <SavingsScreen viewport={context.viewport} state="negative" />
  ),
});

export const missingPrice = defineScreen({
  slug: "missing-price",
  title: "An alternative has no complete price",
  description:
    "Blacksmith lists no size with at least 16 vCPU and 64 GB, so the API gives no Blacksmith cost and no lead. The report shows no savings figure and keeps the costs that it knows.",
  rationale:
    "Without both alternative costs, the cheaper alternative is unknown, so the conservative figure does not show.",
  relatedDocs: related("metering"),
  address,
  render: (_props, context) => (
    <SavingsScreen viewport={context.viewport} state="missing-price" />
  ),
});

export const partialCosts = defineScreen({
  slug: "partial-costs",
  title: "Some machine costs are missing",
  description:
    "Two of the three machines have a monthly cost. The row shows the known part as At least, the total says Incomplete, and no savings figure shows.",
  rationale:
    "metering.md: when an own machine has no monthly cost, the report shows no savings figure and asks for the missing costs. The known part stays, apart from zero.",
  relatedDocs: related("metering"),
  address,
  render: (_props, context) => (
    <SavingsScreen viewport={context.viewport} state="partial-costs" />
  ),
});

export const unknownCosts = defineScreen({
  slug: "unknown-costs",
  title: "Some plan prices are not known",
  description:
    "ciproxy lacks the plan price of some days. The plan row shows the known part as At least, and the notice has no action, because nobody enters a plan price.",
  relatedDocs: related("metering", "billing"),
  address,
  render: (_props, context) => (
    <SavingsScreen viewport={context.viewport} state="unknown-costs" />
  ),
});

export const updating = defineScreen({
  slug: "updating",
  title: "The report still refreshes",
  description:
    "Another refresh of the report ran at the same time, so the API asks the page to read again. The page shows the report and says that it updates.",
  relatedDocs: related("metering"),
  address,
  render: (_props, context) => (
    <SavingsScreen viewport={context.viewport} state="updating" />
  ),
});
