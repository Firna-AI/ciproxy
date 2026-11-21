import { defineScreen } from "@mokly/mokly";

import { relatedMvp } from "../../../../../components/related-docs.ts";
import { orgAddress } from "../../../../../fixtures/org.ts";
import { MachineDetailScreen } from "../../../../../screens/machines/machine-detail.tsx";

export const agentBehind = defineScreen({
  slug: "agent-behind",
  title: "Agent behind the newest release",
  description:
    "Release 0.4.2 of the agent is out, and hetzner-ax42-1 still reports 0.4.1. Health shows the reported release with the newest one beside it and an amber dot. The machine runs jobs as before.",
  rationale:
    "The agent doc says that the dashboard shows whether the agent of each machine is up to date, and the answer B to item 101 adds the newest release to the machine. While a machine is offline, its release is a value of the last contact, so the page shows it in grey and claims neither.",
  relatedDocs: relatedMvp("agent"),
  address: orgAddress("machines/hetzner-ax42-1"),
  render: (_props, context) => (
    <MachineDetailScreen viewport={context.viewport} state="agent-behind" />
  ),
});
