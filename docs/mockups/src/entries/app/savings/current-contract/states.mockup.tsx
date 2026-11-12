import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { SavingsScreen } from "../../../../screens/savings/savings.tsx";

export const report = defineScreen({
  slug: "report",
  title: "Savings, with the facts of the API",
  description:
    "The report of the release as the API gives it. The API gives no tier name and no machine count, so the rows say Own machines and ciproxy plan.",
  rationale:
    "The known-data report stays as the target. This page shows only what the API gives today.",
  relatedDocs: related("metering"),
  address: orgAddress("savings"),
  render: (_props, context) => (
    <SavingsScreen viewport={context.viewport} state="current" />
  ),
});
