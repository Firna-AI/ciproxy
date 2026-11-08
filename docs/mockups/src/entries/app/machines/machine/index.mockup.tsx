import { defineScreen } from "@mokly/mokly";

import { relatedMvp } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { MachineDetailScreen } from "../../../../screens/machines/machine-detail.tsx";

export const machine = defineScreen({
  title: "Machine detail",
  description:
    "hetzner-ax42-1 with what runs in each slot, its warm runner count, health checks, cache use with Clear cache, and the monthly cost that the savings report uses. The name of each running job opens its job page. Its other states are under Machine states.",
  rationale:
    "The answer A to item 102: the warm runner count is the saved count of the machine. Every state of the page shows it, also while the machine drains, is offline, or has no slots, so a change starts from the saved count. The notices describe the state of the machine, and the slots show the warm runners that are ready. The answer B to item 101: the name of the job in a busy slot opens the page of that job.",
  relatedDocs: relatedMvp("agent", "jobRouting", "cache", "metering"),
  address: orgAddress("machines/hetzner-ax42-1"),
  render: (_props, context) => (
    <MachineDetailScreen viewport={context.viewport} state="healthy" />
  ),
});
