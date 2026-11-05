import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../components/paths.ts";
import { related } from "../../../components/related-docs.ts";
import { formatCount, formatMoney } from "../../../fixtures/format.ts";
import { orgAddress } from "../../../fixtures/org.ts";
import { savings as report, yourCost } from "../../../fixtures/savings.ts";
import { SavingsScreen } from "../../../screens/savings/savings.tsx";

const jobs = report.starts.own + report.starts.boat;

export const savings = defineScreen({
  title: "Savings",
  description: `The last 30 days of mokly-ai, all on Team: ${formatCount(jobs)} jobs cost ${formatMoney(yourCost)} with ciproxy (three own machines, Boat overflow, and Team), against ${formatMoney(report.alternatives.blacksmith)} on Blacksmith and ${formatMoney(report.alternatives.github)} on GitHub-hosted runners.`,
  rationale:
    "One hero figure leads against the cheaper alternative, so the claim stays conservative. The bars put the three costs on one axis. The note names the day of the price lists that the report uses.",
  relatedDocs: related("metering"),
  address: orgAddress("savings"),
  useCasePaths: [paths.flowPeakOverflow],
  render: (_props, context) => (
    <SavingsScreen viewport={context.viewport} state="report" />
  ),
});
