import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { SavingsScreen } from "../../../../screens/savings/savings.tsx";

export const report = defineScreen({
  slug: "report",
  title: "Savings with two providers",
  description:
    "A what-if: mokly-ai connected Second provider, the fixture name of a later provider, on 9 November, and it ran the overflow of ciproxy-large since then. The cost table has a row for each provider: Boat overflow with 4,125 jobs and $84.60, and Second provider overflow with 412 jobs and $17.40. The totals and the hero figure stay as in the main report. The chart keeps one series for the providers together, named Boat and Second provider.",
  rationale:
    "The design call of 23 November 2026: the savings report has one overflow place with a row for each provider, because one month can hold the costs of several providers. The chart tokens hold one colour for the overflow, so the providers share one series.",
  relatedDocs: related("metering"),
  address: orgAddress("savings"),
  render: (_props, context) => (
    <SavingsScreen viewport={context.viewport} state="two-providers" />
  ),
});
