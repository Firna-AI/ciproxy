import { defineScreen } from "@mokly/mokly";

import { planDoc, stateSchema } from "../../../../components/state-schema.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import {
  MachineDetailScreen,
  machineStates,
} from "../../../../screens/machines/machine-detail.tsx";

export const machine = defineScreen({
  title: "Machine detail",
  description:
    "hetzner-ax42-1 with what runs in each slot, its warm runner count, health checks, cache use with Clear cache, and the monthly cost that the savings report uses.",
  relatedDocs: [planDoc],
  address: orgAddress("machines/hetzner-ax42-1"),
  propSchema: stateSchema(machineStates),
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
      description:
        "ciproxy lost contact with hetzner-ax42-3 14 minutes ago. Health shows the other checks with their last values, in grey.",
      address: orgAddress("machines/hetzner-ax42-3"),
      props: { state: "offline" },
    },
    {
      slug: "no-slots",
      title: "Cannot run jobs",
      description:
        "A firmware update turned virtualization off on hetzner-ax42-3. The agent connects, but without KVM the machine cannot start a VM for each job, so it has no slots. The notice names the fix, and the machine still counts toward the machines of the tier.",
      address: orgAddress("machines/hetzner-ax42-3"),
      props: { state: "no-slots" },
    },
    {
      slug: "clear-cache",
      title: "Clear cache",
      description:
        "Clear cache deletes every entry of the machine cache of hetzner-ax42-1, for example to drop a bad entry that the machine kept after GitHub deleted it. The GitHub cache keeps its entries.",
      props: { state: "clear-cache" },
    },
  ],
});
