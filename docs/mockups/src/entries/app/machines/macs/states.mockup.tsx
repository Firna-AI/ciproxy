import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { AddMachineScreen } from "../../../../screens/machines/add-machine.tsx";
import { MachinesScreen } from "../../../../screens/machines/machines-list.tsx";

const rationale =
  "The agent doc says that an own machine is an x64 Linux machine or an Apple Silicon Mac, and that the same install command connects both. A Mac counts as one own machine on every tier, at the price of any own machine, as the billing doc says. A Mac uses the slot rule of a Linux machine and runs at most 2 job VMs at a time. A Mac needs macOS 26.7.3 or later, the macOS of the job image. Its 512 GB of disk is a placeholder until the size of the macOS image is measured.";

export const list = defineScreen({
  slug: "list",
  title: "Machines with a Mac",
  description:
    "A what-if of 18 November: mokly-ai added mac-studio-1 as its fourth machine, beside its three AX42 machines. The caption under each name gives the platform, Linux or macOS, with the threads or cores and the memory. mac-studio-1 shows its 6 slots, with one job and one warm runner, its 2 VMs. Four machines pass the three of Team, so the banner offers the extra-machine add-on. The Mac opens its machine page.",
  rationale,
  relatedDocs: related("agent", "jobRouting", "metering"),
  address: orgAddress("machines"),
  render: (_props, context) => (
    <MachinesScreen viewport={context.viewport} state="macs" />
  ),
});

export const add = defineScreen({
  slug: "add",
  title: "A Mac connected",
  description:
    "mac-studio-1 connected with the same install command as a Linux machine. The line under its name gives its cores, memory, virtualization, and 6 slots, and the next line says that it runs the jobs of macOS labels, at most 2 at a time. The note under the command names what a Linux machine and a Mac need. Open Machines shows the list with the Mac.",
  rationale,
  relatedDocs: related("agent", "agentInstall", "metering"),
  address: orgAddress("machines/new"),
  render: (_props, context) => (
    <AddMachineScreen viewport={context.viewport} state="mac-connected" />
  ),
});
