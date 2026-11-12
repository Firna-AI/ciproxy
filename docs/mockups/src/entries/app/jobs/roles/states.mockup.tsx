import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { JobsLiveScreen } from "../../../../screens/jobs/jobs-live.tsx";

const address = orgAddress("jobs");

export const viewer = defineScreen({
  slug: "viewer",
  title: "Jobs, seen by a Viewer",
  description:
    "Jonas Weber, a Viewer, sees the live queue of 26 November at the Boat spend limit, read-only. In place of Raise the limit, he reads Ask an Admin, because only an Admin changes Settings, and Settings is not in his navigation.",
  rationale:
    "Every role sees the jobs of all repositories. A notice keeps its state for everyone, and only the role that may act gets its action.",
  relatedDocs: related("security", "jobRouting", "overflow"),
  address,
  render: (_props, context) => (
    <JobsLiveScreen
      viewport={context.viewport}
      state="spend-limit"
      role="viewer"
    />
  ),
});

export const billing = defineScreen({
  slug: "billing",
  title: "Jobs, seen by a Billing person",
  description:
    "Dana Kim has the Billing role. On 4 September, mokly-ai passed the Free starts, and the banner on Jobs gives her its action, Plan and usage, because she pays. Settings is not in her navigation.",
  rationale:
    "Billing contains Viewer and adds the plan, the add-ons, and the invoices, so the banners of a limit or a payment act for a Billing person on every page.",
  relatedDocs: related("security", "billing", "metering"),
  address,
  render: (_props, context) => (
    <JobsLiveScreen
      viewport={context.viewport}
      state="starts-passed"
      role="billing"
    />
  ),
});
