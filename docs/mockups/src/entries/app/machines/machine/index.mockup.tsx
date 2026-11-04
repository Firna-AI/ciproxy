import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { MachineDetailScreen } from "../../../../screens/machines/machine-detail.tsx";

export const machine = defineScreen({
  title: "Machine detail",
  description:
    "hetzner-ax42-1 with what runs in each slot, its warm runner count, health checks, cache use with Clear cache, and the monthly cost that the savings report uses. Its other states are under Machine states.",
  relatedDocs: related("agent", "jobRouting", "cache", "metering"),
  address: orgAddress("machines/hetzner-ax42-1"),
  render: (_props, context) => (
    <MachineDetailScreen viewport={context.viewport} state="healthy" />
  ),
});
