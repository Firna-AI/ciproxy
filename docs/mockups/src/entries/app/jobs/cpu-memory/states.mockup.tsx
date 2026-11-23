import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { JobDetailScreen } from "../../../../screens/jobs/job-detail.tsx";

const relatedDocs = related("jobUsage", "jobRouting", "securityData");

export const outOfMemory = defineScreen({
  slug: "out-of-memory",
  title: "A step that ran out of memory",
  description:
    "visual diff of mokly #1538 failed on hetzner-ax42-3. In Compare screenshots, the memory reached the top of the 8 GB VM, and the kernel stopped one process. The Out of memory figure, the row of the step, and a red dot on the memory chart show it. The step failed, Upload diff images ran because the job failed, and the post step of actions/setup-node was skipped, so it shows no figures. The meter shows 7.8 of 8 GB, because the kernel of the VM keeps a part of its memory. History opens this page from the visual diff row.",
  rationale:
    "When the kernel stops a process for lack of memory, only that process ends, and the job goes on, so the guest still sends the figures. On Linux, each point counts the processes that the kernel stopped. The table shows the Out of memory column only when a step ran out of memory.",
  relatedDocs,
  address: orgAddress("jobs/48213936"),
  render: (_props, context) => (
    <JobDetailScreen viewport={context.viewport} state="out-of-memory" />
  ),
});

export const macos = defineScreen({
  slug: "macos",
  title: "A macOS job",
  description:
    "A what-if of macOS jobs on 18 November: test (safari) of mokly #1542 ran on mac-studio-1 with the label ciproxy-macos, 2 vCPU and 8 GB. macOS gives no count of the processes that ran out of memory, so the panel has no Out of memory figure, and the table has no such column. Its memory counts app, wired, and compressed memory, as Activity Monitor does, so the memory starts higher than on Linux.",
  rationale:
    "The file cache fills all free memory, so the figures leave it out on both platforms. macOS gives no out-of-memory count, and the page claims none.",
  relatedDocs,
  address: orgAddress("jobs/48213912"),
  render: (_props, context) => (
    <JobDetailScreen viewport={context.viewport} state="macos" />
  ),
});

export const running = defineScreen({
  slug: "running",
  title: "A job that is running",
  description:
    "test (rust) of mokly #1542 while it runs on hetzner-ax42-1, 4m 12s after a warm runner took it. The header says Running, the timeline has no end yet, and Details shows Running for in place of Duration. The panel says that the figures show when the job ends. The job page shows the same job after it passed.",
  rationale:
    "The guest program sends the figures once, with the exit report of the runner, so a running job has no figures yet.",
  relatedDocs,
  address: orgAddress("jobs/48213977"),
  render: (_props, context) => (
    <JobDetailScreen viewport={context.viewport} state="running" />
  ),
});

export const noFigures = defineScreen({
  slug: "no-figures",
  title: "A job without figures",
  description:
    "A what-if: typecheck of mokly #1538 ran on hetzner-ax42-1 with an agent from before the figures, so no figures reached ciproxy. The panel keeps its place and says in one sentence that the figures are not available. A job on an older image, a job with a lost exit report, and a job from before the change look the same. History opens this page from the typecheck row.",
  rationale:
    "A lost report loses the figures, and no queue sends them again, because the figures inform and never decide. The API does not tell the causes apart, so the page names none.",
  relatedDocs,
  address: orgAddress("jobs/48213925"),
  render: (_props, context) => (
    <JobDetailScreen viewport={context.viewport} state="no-figures" />
  ),
});

export const noSteps = defineScreen({
  slug: "no-steps",
  title: "A job without steps",
  description:
    "A what-if: ciproxy could not read the steps of unit (node 24) of mokly-cloud #318 on hetzner-ax42-3, for example after a runner update changed the log of the runner. The panel shows the figures of the job and the chart without step marks, and one sentence in place of the step table. History opens this page from the unit (node 24) row.",
  rationale:
    "The guest reads the steps from the log of the runner, which is not a contract of the runner, and job code can write that log. When the control plane drops the steps of a report, it keeps the figures.",
  relatedDocs,
  address: orgAddress("jobs/48213946"),
  render: (_props, context) => (
    <JobDetailScreen viewport={context.viewport} state="no-steps" />
  ),
});
