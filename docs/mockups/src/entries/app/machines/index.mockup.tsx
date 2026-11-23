import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../components/paths.ts";
import { related } from "../../../components/related-docs.ts";
import { orgAddress } from "../../../fixtures/org.ts";
import { MachinesScreen } from "../../../screens/machines/machines-list.tsx";

export const machines = defineScreen({
  title: "Machines",
  description:
    "The three own machines of mokly-ai on a busy afternoon, with their slots, warm runners, health, and cache. The Overflow panel has a row for each connected provider, here Boat, with its account and its spend against its own limit, and the overflow wait under the rows.",
  rationale:
    "The slot bar shows at a glance how full each machine is and how many warm runners wait, which decides whether the next job starts at once.",
  relatedDocs: related("agent", "jobRouting", "cache", "overflow"),
  address: orgAddress("machines"),
  useCasePaths: [paths.flowPeakOverflow],
  render: (_props, context) => (
    <MachinesScreen viewport={context.viewport} state="steady" />
  ),
});
