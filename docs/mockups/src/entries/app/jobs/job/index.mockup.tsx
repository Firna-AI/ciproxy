import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { planDoc, stateSchema } from "../../../../components/state-schema.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { JobDetailScreen } from "../../../../screens/jobs/job-detail.tsx";

export const job = defineScreen({
  title: "Job detail",
  description:
    "A job that ran on an own machine. A warm runner took it at once, and the cache came from the machine's disk.",
  relatedDocs: [planDoc],
  address: orgAddress("jobs/48213977"),
  propSchema: stateSchema(["own", "boat"]),
  props: { state: "own" },
  render: (props, context) => (
    <JobDetailScreen viewport={context.viewport} state={props.state} />
  ),
  variants: [
    {
      slug: "boat",
      title: "A job that ran on Boat",
      description:
        "The own machines were full, so the job waited 30 seconds and then ran on Boat. The cache came from the GitHub cache.",
      address: orgAddress("jobs/48213902"),
      props: { state: "boat" },
      useCasePaths: [paths.flowPeakOverflow],
    },
  ],
});
