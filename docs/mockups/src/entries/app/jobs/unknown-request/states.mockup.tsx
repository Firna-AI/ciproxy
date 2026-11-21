import { defineScreen } from "@mokly/mokly";

import { relatedMvp } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { JobDetailScreen } from "../../../../screens/jobs/job-detail.tsx";

const rationale =
  "Milestone 21 of the MVP plan: for a running or finished job whose request ciproxy did not save, the API gives no vCPU, memory, or free disk. The current size of the label never takes their place, because an Admin can change a label after its jobs ran. The size of the VM that ran the job comes apart from the request, so it stays known.";

export const ownMachine = defineScreen({
  slug: "own-machine",
  title: "A job with an unknown request",
  description:
    "A what-if of the history of 18 November: ciproxy has no saved request for storybook, which ran on hetzner-ax42-1. In place of a size, the label says Requested size unknown. The VM size comes from the VM that ran the job: 2 vCPU, 8 GB, and a VM disk of 81 GB, the image with its free disk. History opens this page from the storybook row.",
  rationale,
  relatedDocs: relatedMvp("jobRouting", "labels"),
  address: orgAddress("jobs/48213884"),
  render: (_props, context) => (
    <JobDetailScreen viewport={context.viewport} state="unknown-request" />
  ),
});

export const boat = defineScreen({
  slug: "boat",
  title: "A Boat job with an unknown request",
  description:
    "A what-if of the same history: ciproxy has no saved request for docker image, which ran on Boat. The label says Requested size unknown. The Boat size that ran the job stays known: large, 8 vCPU, 16 GB, and 111 GB of disk, which Boat bills. Its caption no longer says that the size fits the label, because the request is unknown. History opens this page from the docker image row.",
  rationale,
  relatedDocs: relatedMvp("jobRouting", "labels", "overflow"),
  address: orgAddress("jobs/48213917"),
  render: (_props, context) => (
    <JobDetailScreen viewport={context.viewport} state="unknown-request-boat" />
  ),
});
