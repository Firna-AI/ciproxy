import { defineScreen } from "@mokly/mokly";

import { related, relatedMvp } from "../../../../../components/related-docs.ts";
import { orgAddress } from "../../../../../fixtures/org.ts";
import { MachineDetailScreen } from "../../../../../screens/machines/machine-detail.tsx";

const docs = related("agent", "jobRouting", "cache", "metering");

export const draining = defineScreen({
  slug: "draining",
  title: "Draining",
  description:
    "The machine finishes its jobs and takes no new ones, for example before maintenance.",
  relatedDocs: docs,
  address: orgAddress("machines/hetzner-ax42-1"),
  render: (_props, context) => (
    <MachineDetailScreen viewport={context.viewport} state="draining" />
  ),
});

export const offline = defineScreen({
  slug: "offline",
  title: "Offline",
  description:
    "ciproxy lost contact with hetzner-ax42-3 14 minutes ago. Health shows the other checks with their last values, in grey.",
  relatedDocs: docs,
  address: orgAddress("machines/hetzner-ax42-3"),
  render: (_props, context) => (
    <MachineDetailScreen viewport={context.viewport} state="offline" />
  ),
});

export const noSlots = defineScreen({
  slug: "no-slots",
  title: "Cannot run jobs",
  description:
    "A firmware update turned virtualization off on hetzner-ax42-3. The agent connects, but without KVM the machine cannot start a VM for each job, so it has no slots. The notice names the fix, and the machine still counts toward the machines of the tier.",
  relatedDocs: docs,
  address: orgAddress("machines/hetzner-ax42-3"),
  render: (_props, context) => (
    <MachineDetailScreen viewport={context.viewport} state="no-slots" />
  ),
});

export const diskLimit = defineScreen({
  slug: "disk-limit",
  title: "Disk limits the jobs",
  description:
    "office-mini-1, a mini PC in the office, joined as the fourth machine. Its 16 threads and 64 GB give 6 slots. After the host reserve, its 256 GB disk leaves 192 GB for jobs, and a job of ciproxy holds 60 GB, so it runs 3 such jobs at a time. Its two warm runners hold 120 GB, and the machine cache has the other 72 GB. The notice gives the reason and the fix. The banner of every page offers the extra-machine add-on, because Team includes three machines.",
  rationale:
    "Design call 34, as the plan defines it: a size fits a machine when the machine has enough free slots and enough free disk. The host reserve keeps a quarter of the disk, each running job holds the disk of its size, and the machine cache uses the rest.",
  relatedDocs: relatedMvp("agent", "jobRouting", "cache"),
  address: orgAddress("machines/office-mini-1"),
  render: (_props, context) => (
    <MachineDetailScreen viewport={context.viewport} state="disk-limit" />
  ),
});

export const clearCache = defineScreen({
  slug: "clear-cache",
  title: "Clear cache",
  description:
    "Clear cache deletes every entry of the machine cache of hetzner-ax42-1, for example to drop a bad entry that the machine kept after GitHub deleted it. The GitHub cache keeps its entries.",
  relatedDocs: docs,
  address: orgAddress("machines/hetzner-ax42-1"),
  render: (_props, context) => (
    <MachineDetailScreen viewport={context.viewport} state="clear-cache" />
  ),
});
