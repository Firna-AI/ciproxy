import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { JobDetailScreen } from "../../../../screens/jobs/job-detail.tsx";
import { JobsHistoryScreen } from "../../../../screens/jobs/jobs-history.tsx";
import { JobsLiveScreen } from "../../../../screens/jobs/jobs-live.tsx";

const rationale =
  "The richer known-data pages stay as the target. These pages show only what the API gives today: no person or date of a suspension, no reason of a Boat refusal, no Boat account address, no total of the day, no tier name, no routing reason, and no fit of a Boat size.";

export const suspended = defineScreen({
  slug: "suspended",
  title: "The App is suspended, with the facts of the API",
  description:
    "The suspension of 19 November as the API gives it: the organization and its GitHub address, with no person and no date. Open GitHub opens the install of mokly-ai.",
  rationale,
  relatedDocs: related("appRemoval", "jobRouting"),
  address: orgAddress("jobs"),
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="suspended-current" />
  ),
});

export const boatRefused = defineScreen({
  slug: "boat-refused",
  title: "Boat refuses sandboxes, with the facts of the API",
  description:
    "The refusal of 25 November as the API gives it: the time when ciproxy tries Boat again, with no reason and no address of the Boat account, so the notice has no action.",
  rationale,
  relatedDocs: related("overflow", "jobRouting"),
  address: orgAddress("jobs"),
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="refused-current" />
  ),
});

export const history = defineScreen({
  slug: "history",
  title: "History, with the facts of the API",
  description:
    "The history of 18 November as the API gives it: every job that matches, the count of the list, and the window of the plan in days. A job that no runner took says No runner, and a job with no finish time says Unknown. Rows open the job pages of this folder and of the job states.",
  rationale,
  relatedDocs: related("jobRouting", "billing", "metering"),
  address: orgAddress("jobs/history"),
  render: (_props, context) => (
    <JobsHistoryScreen viewport={context.viewport} state="current" />
  ),
});

export const ownJob = defineScreen({
  slug: "own-job",
  title: "A job on an own machine, with the facts of the API",
  description:
    "test (rust) as the API gives it: the times when GitHub queued it, when a runner took it, and when it passed. The API gives no routing reason yet, so the page says that it is not available.",
  rationale,
  relatedDocs: related("jobRouting", "labels", "cache", "metering"),
  address: orgAddress("jobs/48213977"),
  render: (_props, context) => (
    <JobDetailScreen viewport={context.viewport} state="current-own" />
  ),
});

export const boatJob = defineScreen({
  slug: "boat-job",
  title: "A job on Boat, with the facts of the API",
  description:
    "e2e (chromium) 3/4 as the API gives it. The VM size names the Boat size and says that Boat bills it, with no claim that the size fits the label, because the API gives no fit.",
  rationale,
  relatedDocs: related("jobRouting", "overflow", "labels"),
  address: orgAddress("jobs/48213902"),
  render: (_props, context) => (
    <JobDetailScreen viewport={context.viewport} state="current-boat" />
  ),
});
