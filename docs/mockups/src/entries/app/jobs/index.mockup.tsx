import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../components/paths.ts";
import { planDoc } from "../../../components/state-schema.ts";
import { orgAddress } from "../../../fixtures/org.ts";
import { JobsLiveScreen } from "../../../screens/jobs/jobs-live.tsx";

export const jobs = defineScreen({
  title: "Jobs",
  description:
    "The live queue on a busy Wednesday afternoon: 21 jobs run, 15 on the three own machines and 6 on Boat from a burst a few minutes earlier. Nothing waits, because warm runners are free.",
  rationale:
    "The figures lead with what a team asks first: is anything waiting, and are the own machines in use.",
  relatedDocs: [planDoc],
  address: orgAddress("jobs"),
  useCasePaths: [paths.flowPeakOverflow],
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="steady" />
  ),
});
