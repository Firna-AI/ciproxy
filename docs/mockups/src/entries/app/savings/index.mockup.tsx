import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../components/paths.ts";
import { related } from "../../../components/related-docs.ts";
import { orgAddress } from "../../../fixtures/org.ts";
import { SavingsScreen } from "../../../screens/savings/savings.tsx";

export const savings = defineScreen({
  title: "Savings",
  description:
    "The last 30 days of mokly-ai, all on Team: 17,772 jobs cost $309.20 with ciproxy (three own machines, Boat overflow, and Team), against $698 on Blacksmith and $1,045 on GitHub-hosted runners.",
  rationale:
    "One hero figure leads against the cheaper alternative, so the claim stays conservative. The bars put the three costs on one axis. The note names the day of the price lists that the report uses.",
  relatedDocs: related("metering"),
  address: orgAddress("savings"),
  useCasePaths: [paths.flowPeakOverflow],
  render: (_props, context) => (
    <SavingsScreen viewport={context.viewport} state="report" />
  ),
});
