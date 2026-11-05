import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { related } from "../../../../components/related-docs.ts";
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
  relatedDocs: related("jobRouting", "cache", "overflow", "metering"),
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
  ],
});
