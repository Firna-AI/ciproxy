import { defineScreen } from "@mokly/mokly";

import { relatedMvp } from "../../../../../components/related-docs.ts";
import { orgAddress } from "../../../../../fixtures/org.ts";
import { MachineDetailScreen } from "../../../../../screens/machines/machine-detail.tsx";

const address = orgAddress("machines/hetzner-ax42-1");

const rationale =
  "The answer E to item 118: each producer marks a figure that it did not measure, and the page shows that figure as Unknown, never as a zero or a share. A measured zero still shows as 0, as office-mini-1 shows under Disk limits the jobs.";

export const cacheUnknown = defineScreen({
  slug: "cache-unknown",
  title: "Machine cache unknown",
  description:
    "hetzner-ax42-1 did not measure its machine cache for its last report, for example after its cache failed to load its saved counts. Cache disk and Cache hits, 7 days say Unknown, with no meter bar, and the note says that they show after the machine measures them. The Docker disks keep their measured figures. Clear cache stays, because a clear needs no figures. On Machines, the row of hetzner-ax42-1 opens this page.",
  rationale,
  relatedDocs: relatedMvp("cache", "agent"),
  address,
  render: (_props, context) => (
    <MachineDetailScreen viewport={context.viewport} state="cache-unknown" />
  ),
});

export const dockerUnknown = defineScreen({
  slug: "docker-unknown",
  title: "Docker disks unknown",
  description:
    "The agent of hetzner-ax42-1 could not read its Docker disks for its last report. Docker disks say Unknown, and the note says that they show after the machine measures them. The machine cache keeps its measured figures. Without the Docker figures, the free disk for jobs is unknown too, so the page shows the machine as for any unknown capacity, with no disk limit.",
  rationale: `${rationale} No reader counts unknown Docker disks as zero bytes: ciproxy keeps the machine in its labels, but starts no new runner there until the agent measures its Docker disks again.`,
  relatedDocs: relatedMvp("dockerDisks", "jobRouting", "agent"),
  address,
  render: (_props, context) => (
    <MachineDetailScreen viewport={context.viewport} state="docker-unknown" />
  ),
});

export const bothUnknown = defineScreen({
  slug: "both-unknown",
  title: "Cache and Docker disks unknown",
  description:
    "hetzner-ax42-1 measured neither its machine cache nor its Docker disks for its last report. Each figure says Unknown, and one note names them all. The capacity is unknown, as when only the Docker disks are unknown. Clear cache stays. Roles shows the same page to a Billing person and a Viewer.",
  rationale:
    "The machine cache and the Docker disks are two parts of the report, and each is measured or unavailable on its own. So the page shows the figures of each part apart, and one note covers every part that shows Unknown.",
  relatedDocs: relatedMvp("cache", "dockerDisks", "agent"),
  address,
  render: (_props, context) => (
    <MachineDetailScreen viewport={context.viewport} state="both-unknown" />
  ),
});
