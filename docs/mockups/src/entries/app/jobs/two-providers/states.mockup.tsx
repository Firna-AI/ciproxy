import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { JobDetailScreen } from "../../../../screens/jobs/job-detail.tsx";
import { JobsHistoryScreen } from "../../../../screens/jobs/jobs-history.tsx";

export const secondProvider = defineScreen({
  slug: "second-provider",
  title: "A job that ran on the next provider",
  description:
    "A what-if with two providers on 18 November: the order of ciproxy is Your machines, Boat, then Second provider, the fixture name of a later provider. docker image of mokly-cloud waited 30 seconds for a free own slot. Boat refused new sandboxes, because the start limit of the Boat plan was full, so the job started on Second provider at once. The reason and the timeline name each place that the job passed. Second provider ran it on standard-2, its cheapest size that fits the 8 GB of the label and its VM disk of 81 GB, and Second provider bills that size. The history of the same what-if opens this page.",
  rationale:
    "The user's decision of 23 November 2026: a job goes to the next place of its order when the current place cannot take it now, with no extra wait between providers. The overflow wait applies only when a job leaves the own machines. The page names the provider that ran the job and why the job went there.",
  relatedDocs: related("jobRouting", "labels", "overflow", "metering"),
  address: orgAddress("jobs/48214410"),
  render: (_props, context) => (
    <JobDetailScreen viewport={context.viewport} state="second-provider" />
  ),
});

export const history = defineScreen({
  slug: "history",
  title: "History with two providers",
  description:
    "The finished jobs of 18 November in the same what-if. docker image ran on Second provider, and the other jobs on the own machines and on Boat. The place filter offers All, Own machines, and one choice for each provider that ran jobs in the window: Boat and Second provider. The row of docker image opens its job page.",
  rationale:
    "The design call of 23 November 2026: the job history filter has one choice for each provider that ran jobs. A provider that ran no job in the window gets no choice.",
  relatedDocs: related("jobRouting", "overflow"),
  address: orgAddress("jobs/history"),
  render: (_props, context) => (
    <JobsHistoryScreen viewport={context.viewport} state="two-providers" />
  ),
});
