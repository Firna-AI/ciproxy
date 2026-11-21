import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../../components/related-docs.ts";
import { orgAddress } from "../../../../../fixtures/org.ts";
import { MachineDetailScreen } from "../../../../../screens/machines/machine-detail.tsx";

export const clearRequested = defineScreen({
  slug: "clear-requested",
  title: "Clear cache requested",
  description:
    "Cal Moore confirmed Clear cache for hetzner-ax42-1, and the connected agent took the command. The notice says that the machine clears its cache now. The cache use still shows the last report until the next report of the agent.",
  rationale:
    "The page can show the effect of Clear cache only after the next report of the agent, so a notice confirms the action at once. The GitHub cache and the Docker disks keep their data.",
  relatedDocs: related("cache", "agent"),
  address: orgAddress("machines/hetzner-ax42-1"),
  render: (_props, context) => (
    <MachineDetailScreen viewport={context.viewport} state="clear-requested" />
  ),
});

export const savedOffline = defineScreen({
  slug: "saved-offline",
  title: "Saved until the machine connects",
  description:
    "The connection of hetzner-ax42-1 dropped 25 seconds ago, before ciproxy marks the machine offline after 60 seconds, and Cal Moore selected Drain. ciproxy saved the drain, and the machine gets it when it connects again. The page shows the machine as draining, with Stop draining.",
  rationale:
    "A command that finds no connected agent applies when the agent connects again, so the page says that the change is saved and when it applies.",
  relatedDocs: related("agent", "jobRouting"),
  address: orgAddress("machines/hetzner-ax42-1"),
  render: (_props, context) => (
    <MachineDetailScreen viewport={context.viewport} state="saved-offline" />
  ),
});
