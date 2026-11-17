import { defineScreen } from "@mokly/mokly";

import { relatedMvp } from "../../../../../components/related-docs.ts";
import { orgAddress } from "../../../../../fixtures/org.ts";
import { MachineDetailScreen } from "../../../../../screens/machines/machine-detail.tsx";

const address = orgAddress("machines/hetzner-ax42-1");

const rationale =
  "The answer E to item 118: a figure that the machine did not measure shows as Unknown. The confirmation of each page with unknown figures names no size of the cache entries, also when the machine measured its machine cache. Clear cache keeps its Admin rule, because a clear needs no figures. The confirmation opens over the page of the same moment, and Cancel returns to that page. The catalogue has no page after this clear, so Clear cache opens no screen here.";

export const clearCacheUnknown = defineScreen({
  slug: "cache-unknown",
  title: "Clear cache, machine cache unknown",
  description:
    "Cal Moore selects Clear cache on Machine cache unknown. The confirmation opens over the same page of hetzner-ax42-1, where Cache disk and Cache hits, 7 days say Unknown. It names no size of the cache entries, because the machine did not measure one.",
  rationale,
  relatedDocs: relatedMvp("cache", "agent"),
  address,
  render: (_props, context) => (
    <MachineDetailScreen
      viewport={context.viewport}
      state="clear-cache-unknown"
    />
  ),
});

export const clearDockerUnknown = defineScreen({
  slug: "docker-unknown",
  title: "Clear cache, Docker disks unknown",
  description:
    "Cal Moore selects Clear cache on Docker disks unknown. The confirmation opens over the same page of hetzner-ax42-1, where Docker disks say Unknown and the machine cache keeps its measured figures. It names no size of the cache entries, as on every page with unknown figures. Clear cache never deletes a Docker disk.",
  rationale,
  relatedDocs: relatedMvp("cache", "dockerDisks", "agent"),
  address,
  render: (_props, context) => (
    <MachineDetailScreen
      viewport={context.viewport}
      state="clear-docker-unknown"
    />
  ),
});

export const clearBothUnknown = defineScreen({
  slug: "both-unknown",
  title: "Clear cache, cache and Docker disks unknown",
  description:
    "Cal Moore selects Clear cache on Cache and Docker disks unknown. The confirmation opens over the same page of hetzner-ax42-1, where each figure says Unknown. It names no size of the cache entries.",
  rationale,
  relatedDocs: relatedMvp("cache", "dockerDisks", "agent"),
  address,
  render: (_props, context) => (
    <MachineDetailScreen
      viewport={context.viewport}
      state="clear-both-unknown"
    />
  ),
});
