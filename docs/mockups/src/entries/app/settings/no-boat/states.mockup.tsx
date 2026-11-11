import { defineScreen } from "@mokly/mokly";

import { relatedMvp } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { LabelsScreen } from "../../../../screens/settings/labels.tsx";

const address = orgAddress("settings/labels");

const rationale =
  "The overflow doc says that without Boat, jobs run on own machines only. The answer B to item 101: the placement of a label comes from the API, which knows when a workspace has no Boat account. Without one, ciproxy checks no Boat size, so the page never says that a Boat size does not fit.";

export const labels = defineScreen({
  slug: "labels",
  title: "Labels without Boat",
  description:
    "mokly-ai skipped the Boat step of setup. Each label fits the three machines, so its jobs run on the machines only, and the list says Your machines only. The jobs of 30 days and Edit stay as with Boat; Edit of ciproxy-8vcpu opens its dialog.",
  rationale,
  relatedDocs: relatedMvp("labels", "overflow"),
  address,
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="no-boat" />
  ),
});

export const editLabel = defineScreen({
  slug: "edit-label",
  title: "Edit a label without Boat",
  description:
    "The dialog of ciproxy-8vcpu in the same workspace. The line under the size keeps the fit of the machines, 1 job of this size at a time on each, and says that Boat is not connected. Run on the largest Boat size names no size, because no Boat account gives one.",
  rationale,
  relatedDocs: relatedMvp("labels", "overflow"),
  address,
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="no-boat-edit" />
  ),
});
