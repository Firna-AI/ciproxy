import { defineScreen } from "@mokly/mokly";

import { relatedMvp } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { LabelsScreen } from "../../../../screens/settings/labels.tsx";

const address = orgAddress("settings/labels");

const rationale =
  "The overflow doc says that without a provider, jobs run on own machines only. The answer B to item 101: the placement of a label comes from the API, which knows when a workspace has no connected provider. A provider that the workspace never connected is in no label order, so the page never says that a provider size does not fit.";

export const labels = defineScreen({
  slug: "labels",
  title: "Labels with no provider",
  description:
    "mokly-ai skipped the provider step of setup and never connected a provider, so the order of each label holds the own machines only. Each label fits the three machines, so the list says Your machines only. The jobs of 30 days and Edit stay as with a provider; Edit of ciproxy-large opens its dialog.",
  rationale,
  relatedDocs: relatedMvp("labels", "overflow"),
  address,
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="no-provider" />
  ),
});

export const editLabel = defineScreen({
  slug: "edit-label",
  title: "Edit a label with no provider",
  description:
    "The dialog of ciproxy-large in the same workspace. The line under the size keeps the fit of the machines, 1 job of this size at a time on each. The order holds Your machines only, and Add a place says that a provider joins after it connects under Overflow. When nothing fits keeps both choices; Run on the largest size says to add a provider to the order first.",
  rationale,
  relatedDocs: relatedMvp("labels", "overflow"),
  address,
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="no-provider-edit" />
  ),
});
