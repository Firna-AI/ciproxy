import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { related, relatedMvp } from "../../../../components/related-docs.ts";
import { stateSchema } from "../../../../components/state-schema.ts";
import { jobDetailStates } from "../../../../fixtures/job-details.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { JobDetailScreen } from "../../../../screens/jobs/job-detail.tsx";

export const job = defineScreen({
  title: "Job detail",
  description:
    "A job that ran on an own machine. A warm runner took it at once. The job VM had the size of the label ciproxy, and the machine cache handled the restores, so the page shows their counts.",
  rationale:
    "The label and the VM size that ran say what the job got. The cache counts come from the agent, so only a job whose restores the machine cache handled shows them.",
  relatedDocs: related("jobRouting", "labels", "cache", "overflow", "metering"),
  address: orgAddress("jobs/48213977"),
  propSchema: stateSchema(jobDetailStates),
  props: { state: "own" },
  render: (props, context) => (
    <JobDetailScreen viewport={context.viewport} state={props.state} />
  ),
  variants: [
    {
      slug: "boat",
      title: "A job that ran on Boat",
      description:
        "The own machines were full, so the job waited 30 seconds and then ran on Boat. Boat ran it on large, 8 vCPU and 16 GB, because it is the cheapest Boat size that fits the 8 GB of the label ciproxy and its VM disk of 82 GB, the current image with 20 GB of free disk: default has the memory, but only 51 GB of disk. Boat bills that size. The page shows only the cache source, the GitHub cache, because only the machine cache counts the restores of a job.",
      address: orgAddress("jobs/48213902"),
      props: { state: "boat" },
      useCasePaths: [paths.flowPeakOverflow],
    },
    {
      slug: "fallback",
      title: "A job that fell back to the GitHub cache",
      description:
        "A warm runner on hetzner-ax42-2 took the job, but the cache of the machine was not available, so the job used the GitHub cache. The page shows only the cache source, as for a job on Boat.",
      address: orgAddress("jobs/48213955"),
      props: { state: "fallback" },
    },
    {
      slug: "mixed-cache",
      title: "A job that restored from both caches",
      description:
        "eval (agents) of mokly-labs, the second organization of the workspace, ran on hetzner-ax42-2. The machine cache handled its 4 restores: some entries were on the machine, and the rest came from the GitHub cache, where jobs on the other machines had saved them. The page shows the counts and names both caches.",
      rationale:
        "Design call 44: a job that restored from both caches shows This machine and the GitHub cache, because one of the two sources would be false for some of its entries.",
      relatedDocs: relatedMvp("jobRouting", "labels", "cache", "metering"),
      address: orgAddress("jobs/48213941"),
      props: { state: "mixed-cache" },
    },
    {
      slug: "no-restores",
      title: "A job that restored no entry",
      description:
        "fmt checks the formatting of mokly and uses no cache. A warm runner on hetzner-ax42-2 ran it, and it restored no entry, so the page says No entries restored and names no cache source.",
      rationale:
        "A design call of the docs work of Milestone 2: a job that restored no entry shows No entries restored, which is true for every such job.",
      relatedDocs: relatedMvp("jobRouting", "labels", "cache", "metering"),
      address: orgAddress("jobs/48213928"),
      props: { state: "no-restores" },
    },
  ],
});
