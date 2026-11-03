import { defineScreen } from "@mokly/mokly";

import { planDoc, stateSchema } from "../../../../components/state-schema.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { MachineDetailScreen } from "../../../../screens/machines/machine-detail.tsx";

export const machine = defineScreen({
  title: "Machine detail",
  description:
    "hetzner-ax42-1 with what runs in each slot, its warm runner count, health checks, cache use, and the monthly cost that the savings report uses.",
  relatedDocs: [planDoc],
  address: orgAddress("machines/hetzner-ax42-1"),
  propSchema: stateSchema(["healthy", "draining", "offline"]),
  props: { state: "healthy" },
  render: (props, context) => (
    <MachineDetailScreen viewport={context.viewport} state={props.state} />
  ),
  variants: [
    {
      slug: "draining",
      title: "Draining",
      description:
        "The machine finishes its jobs and takes no new ones, for example before maintenance.",
      props: { state: "draining" },
    },
    {
      slug: "offline",
      title: "Offline",
      description: "hetzner-ax42-3 stopped sending heartbeats 14 minutes ago.",
      address: orgAddress("machines/hetzner-ax42-3"),
      props: { state: "offline" },
    },
  ],
});
