import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { JobsLiveScreen } from "../../../../screens/jobs/jobs-live.tsx";

const address = orgAddress("jobs");

export const boatRefused = defineScreen({
  slug: "boat-refused",
  title: "Boat refuses new sandboxes",
  description:
    "A peak on 25 November fills the start limit of the Boat plan. Boat refuses new sandboxes until 10:13, so 9 jobs wait past the 30 seconds. Each one takes the first free slot on an own machine, and ciproxy tries Boat again.",
  rationale:
    "A refused start is a limit of the Boat account, not of ciproxy, so the recovery is on Boat. The jobs that run keep running.",
  relatedDocs: related("overflow", "jobRouting"),
  address,
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="boat-refused" />
  ),
});

export const nothingFits = defineScreen({
  slug: "nothing-fits",
  title: "Jobs of a label that nothing fits",
  description:
    "Two load tests use ciproxy-32vcpu, which no own machine and no Boat size fits. The label is set to wait when nothing fits, so the jobs stay queued in GitHub. Edit label opens its dialog.",
  relatedDocs: related("overflow", "jobRouting"),
  address,
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="nothing-fits" />
  ),
});
