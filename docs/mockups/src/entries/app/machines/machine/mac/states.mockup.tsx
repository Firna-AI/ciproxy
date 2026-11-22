import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../../components/related-docs.ts";
import { orgAddress } from "../../../../../fixtures/org.ts";
import { MachineDetailScreen } from "../../../../../screens/machines/machine-detail.tsx";

const rationale =
  "The job routing doc says that a Mac uses the slot rule of a Linux machine, and that it runs at most 2 job VMs at a time, because Apple allows at most 2 macOS VMs at a time on one Mac. A Mac runs the jobs of macOS labels only. The health check names virtualization on both platforms: KVM on Linux, and Virtualization.framework on a Mac. The macOS version that a Mac needs is the macOS of the job image, macOS 26.7.3.";

const relatedDocs = related("agent", "jobRouting", "metering");

export const macStudio = defineScreen({
  slug: "mac-studio",
  title: "A Mac",
  description:
    "A what-if of 18 November: mac-studio-1, a Mac Studio with 16 cores and 64 GB, joined as the fourth machine, beside the three AX42 machines. The subtitle names its platform, macOS. Its memory and cores give 6 slots, but a Mac runs at most 2 VMs at once: one job of ciproxy-macos and one warm runner. So its other 4 slots say that the Mac runs its 2 VMs. Health names Virtualization, not KVM. The Cache panel has no Docker disks row, because a macOS VM runs no Docker. Four machines pass the three of Team, so the banner offers the extra-machine add-on.",
  rationale,
  relatedDocs,
  address: orgAddress("machines/mac-studio-1"),
  render: (_props, context) => (
    <MachineDetailScreen viewport={context.viewport} state="mac" />
  ),
});

export const oneSlot = defineScreen({
  slug: "one-slot",
  title: "A Mac with 1 slot",
  description:
    "A second what-if: mac-mini-1, a Mac mini with 10 cores and 16 GB, joined as the fourth machine in place of mac-studio-1. After the host reserve, its memory leaves room for 1 slot, so it runs 1 job at a time, not 2. The notice says why and that a Mac with more memory runs up to 2 jobs. Its one slot runs a job of ciproxy-macos, so no warm runner is ready.",
  rationale,
  relatedDocs,
  address: orgAddress("machines/mac-mini-1"),
  render: (_props, context) => (
    <MachineDetailScreen viewport={context.viewport} state="mac-one-slot" />
  ),
});

export const oldMacos = defineScreen({
  slug: "old-macos",
  title: "A Mac with an old macOS",
  description:
    "mac-mini-1 when its macOS is older than the macOS of the job image. A macOS VM cannot run a newer macOS than its host, so the Mac cannot start VMs and connects with no slots. Health shows Virtualization as not available. The notice names the fix, an update to macOS 26.7.3 or later, and an Admin gets the install command to run again. Jobs of macOS labels wait until a Mac has a free slot.",
  rationale,
  relatedDocs: related("agent", "agentInstall", "jobRouting"),
  address: orgAddress("machines/mac-mini-1"),
  render: (_props, context) => (
    <MachineDetailScreen
      viewport={context.viewport}
      state="mac-no-virtualization"
    />
  ),
});
