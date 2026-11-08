import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { JobsLiveScreen } from "../../../../screens/jobs/jobs-live.tsx";

const address = orgAddress("jobs");

const rationale =
  "The API gives the fit of each waiting label and the Boat facts. The notice names only those facts, and the action of a setting goes to an Admin.";

export const unknownFit = defineScreen({
  slug: "unknown-fit",
  title: "The fit of a label is unknown",
  description:
    "Monday 23 November: a machine report is missing, so the API gives no fit for ciproxy-8vcpu and no count of free own slots. The figure says Unknown, not 0, and the notice claims no fit and no lack of fit.",
  rationale,
  relatedDocs: related("labels", "jobRouting"),
  address,
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="unknown-fit" />
  ),
});

export const largestBoat = defineScreen({
  slug: "largest-boat",
  title: "A label starts on the largest Boat size",
  description:
    "Friday 27 November: ciproxy-32vcpu now runs on the largest Boat size when nothing fits. Two load tests wait for their sandbox, and the notice says where the jobs of the label run.",
  rationale:
    "labels.md: the live queue shows where the jobs of each label run now. The largest size can be smaller than the label, so the notice names both.",
  relatedDocs: related("labels", "overflow", "jobRouting"),
  address,
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="largest-boat" />
  ),
});

export const boatKey = defineScreen({
  slug: "boat-key",
  title: "Boat rejected the key",
  description:
    "Boat rejects the key of the account. Jobs run on own machines only, and six wait for a slot. Connect a new key opens the overflow settings.",
  rationale,
  relatedDocs: related("overflow", "jobRouting"),
  address,
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="boat-key" />
  ),
});

export const boatBilling = defineScreen({
  slug: "boat-billing",
  title: "Boat needs billing",
  description:
    "Boat asks for billing on its account before it starts more sandboxes. The fix is on Boat, and the API gives no address of the Boat account, so the notice has no action.",
  rationale,
  relatedDocs: related("overflow", "jobRouting"),
  address,
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="boat-billing" />
  ),
});

export const overflowOff = defineScreen({
  slug: "overflow-off",
  title: "Boat overflow is off",
  description:
    "Every own slot runs a job and six jobs wait. Use Boat when your machines are busy is off, so the jobs start when a slot is free. Turn on Boat opens the overflow settings.",
  rationale,
  relatedDocs: related("jobRouting", "overflow"),
  address,
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="overflow-off" />
  ),
});
