import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../../components/related-docs.ts";
import { orgAddress } from "../../../../../fixtures/org.ts";
import { MachineDetailScreen } from "../../../../../screens/machines/machine-detail.tsx";

export const noRoom = defineScreen({
  slug: "no-room",
  title: "No room for a slot",
  description:
    "office-pc-1, an older office PC with 4 threads and 8 GB of memory, joined as the fourth machine. KVM works, but after the host reserve its memory leaves no room for one slot of 2 vCPU and 8 GB, so the machine connects with no slots. The notice says why and names the fix. The machine still counts toward the machines of the tier, so the banner of every page offers the extra-machine add-on.",
  rationale:
    "The agent doc says that a machine without room for one slot after the host reserve connects with no slots, and that the dashboard says why, as it does for a machine without KVM.",
  relatedDocs: related("agent", "jobRouting", "metering"),
  address: orgAddress("machines/office-pc-1"),
  render: (_props, context) => (
    <MachineDetailScreen viewport={context.viewport} state="no-room" />
  ),
});
