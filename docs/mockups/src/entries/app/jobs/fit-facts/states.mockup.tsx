import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { JobsLiveScreen } from "../../../../screens/jobs/jobs-live.tsx";

const address = orgAddress("jobs");

const rationale =
  "The API says that no permitted place fits a label, and which places the label allows, but not which resource is short. So the notice names the full request with fits, the places that the label forbids, and a way forward. The known-data scene of Why jobs wait names the short resources of the story.";

export const nothingFitsRequest = defineScreen({
  slug: "nothing-fits-request",
  title: "No permitted place fits a label",
  description:
    "Two load tests of ciproxy-32vcpu wait. No machine of yours and no Boat size fits a job of 32 vCPU, 128 GB of memory, and 200 GB of free disk. The label still waits when nothing fits, so the notice also offers the largest Boat size. Edit label opens the label for an Admin.",
  rationale,
  relatedDocs: related("labels", "jobRouting"),
  address,
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="nothing-fits-request" />
  ),
});

export const boatForbidden = defineScreen({
  slug: "boat-forbidden",
  title: "A label that does not run on Boat",
  description:
    "An Admin turned off Boat for ciproxy-32vcpu. No machine of yours fits a job of its request, and the label does not run on Boat, so the notice offers no Boat size. Edit label opens the label for an Admin.",
  rationale,
  relatedDocs: related("labels", "jobRouting"),
  address,
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="boat-forbidden" />
  ),
});
