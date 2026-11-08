import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { JobsLiveScreen } from "../../../../screens/jobs/jobs-live.tsx";

const address = orgAddress("jobs");

const rationale =
  "labels.md: the live queue shows where the jobs of each label run now. The API gives the fit of each waiting label, so a notice promises a place and a time only for a label whose facts confirm them.";

export const boatAtOnce = defineScreen({
  slug: "boat-at-once",
  title: "A label that only Boat runs starts at once",
  description:
    "Every own slot runs a job. Six jobs of ciproxy wait for a free slot, and each one starts on Boat when it has waited 30 seconds. No own machine runs ciproxy-16vcpu, so its two jobs start on Boat at once, with no overflow wait. A label that Boat cannot run, or whose Boat facts the API leaves out, drops out of the Boat sentence.",
  rationale:
    "job-routing.md: a job that no own machine can run starts on Boat at once. The notice promises Boat only for a label that Boat can start now.",
  relatedDocs: related("jobRouting", "labels", "overflow"),
  address,
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="boat-at-once" />
  ),
});

export const spendUncounted = defineScreen({
  slug: "spend-uncounted",
  title: "Boat costs are not counted yet",
  description:
    "Every own slot runs a job and six jobs wait. The API gives no safe count of the Boat costs, so Boat starts no new sandbox, and the notice promises no Boat start.",
  rationale:
    "overflow.md: when a start has no safe bound, new starts wait until that cost resolves. The API then leaves out whether the spend reached the limit.",
  relatedDocs: related("overflow", "jobRouting"),
  address,
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="spend-uncounted" />
  ),
});

export const limitBoatOnly = defineScreen({
  slug: "limit-boat-only",
  title: "The Boat limit holds a label that only Boat runs",
  description:
    "Boat reached its spend limit. Six jobs of ciproxy wait for a free own slot. No own machine runs ciproxy-16vcpu, so its two jobs cannot take a free slot, and the notice says that only the jobs whose label fits your machines wait there. Raise the limit opens the overflow settings.",
  rationale,
  relatedDocs: related("overflow", "labels", "jobRouting"),
  address,
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="limit-boat-only" />
  ),
});
