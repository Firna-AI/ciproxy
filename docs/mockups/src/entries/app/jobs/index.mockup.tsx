import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../components/paths.ts";
import { relatedMvp } from "../../../components/related-docs.ts";
import { orgAddress } from "../../../fixtures/org.ts";
import { JobsLiveScreen } from "../../../screens/jobs/jobs-live.tsx";

export const jobs = defineScreen({
  title: "Jobs",
  description:
    "The live queue of the workspace mokly-ai on a busy Wednesday afternoon: 21 jobs of its two organizations run, 15 on the three own machines and 6 on Boat from a burst a few minutes earlier. Each row names the organization of its repository, such as mokly-labs/eval-runner. Nothing waits, because warm runners are free.",
  rationale:
    "The figures lead with what a person asks first: is anything waiting, and are the own machines in use. A machine runs the jobs of every organization of the workspace, as the answer to item 48 says.",
  relatedDocs: relatedMvp("jobRouting", "overflow", "workspaces"),
  address: orgAddress("jobs"),
  useCasePaths: [paths.flowPeakOverflow],
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="steady" />
  ),
});
