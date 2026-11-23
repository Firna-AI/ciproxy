import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { LabelsScreen } from "../../../../screens/settings/labels.tsx";

const address = orgAddress("settings/labels");
const docs = related("labels", "jobRouting", "overflow");

const rationale =
  "The user's decisions of 23 November 2026: each label sets its order of places, the own machines and each provider, and a label can put a provider before the own machines or leave them out. A job goes to the next place of its order when the current place cannot take it now, with no extra wait between providers. The overflow wait applies only when a job leaves the own machines.";

export const emptyOrder = defineScreen({
  slug: "empty-order",
  title: "An empty order",
  description:
    "An Admin removed both places of ciproxy-medium. The order has no place, so the jobs of the label wait, and a notice says so. The line under the size says that your machines are not in the order. Add a place offers Your machines and Boat. The list behind the dialog says Waits: no place for the label.",
  rationale: `${rationale} ciproxy never rejects a label on save, so an empty order saves, as a label with both old switches off did.`,
  relatedDocs: docs,
  address,
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="empty-order" />
  ),
});

export const boatFirst = defineScreen({
  slug: "boat-first",
  title: "Boat before the own machines",
  description:
    "An Admin moved Boat before the own machines for ciproxy-large, so its long jobs leave the own slots to short ones. Boat runs the label on xlarge. When Boat cannot take a job, it goes to the own machines at once and waits there for a free slot, so their row names no overflow wait. The list behind the dialog says Boat, then your machines.",
  rationale,
  relatedDocs: docs,
  address,
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="boat-first" />
  ),
});

export const twoProviders = defineScreen({
  slug: "two-providers",
  title: "Labels with two providers",
  description:
    "A what-if: mokly-ai connected Second provider, the fixture name of a later provider, on 9 November. It went last in the order of each Linux label, and an Admin moved it before Boat for ciproxy-large. Where jobs run names each place of the order that can run the label now, in order.",
  rationale: `${rationale} A provider that the workspace connects for the first time goes last in the order of each Linux label.`,
  relatedDocs: docs,
  address,
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="two-providers" />
  ),
});

export const twoProvidersEdit = defineScreen({
  slug: "two-providers-edit",
  title: "Edit the order with two providers",
  description:
    "The dialog of ciproxy-large in the same what-if. Its order is Your machines, Second provider, then Boat. Each row says what the place does with a job of the size: a job waits up to 30 seconds for a free own slot, then starts on Second provider standard-8, and Boat xlarge comes next. A grab handle moves a place, and Remove takes it out. Every place is in the order, so Add a place shows nothing.",
  rationale,
  relatedDocs: docs,
  address,
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="two-providers-edit" />
  ),
});

export const boatUnavailable = defineScreen({
  slug: "boat-unavailable",
  title: "The order while a provider is unavailable",
  description:
    "The dialog of ciproxy in the what-if with two providers, while the deployment has no settings for Boat. Its order is Your machines, Boat, then Second provider. The row of Boat says that Boat is unavailable right now, so jobs skip it, and a job that leaves the own machines starts on Second provider standard-2. Boat stays in the order, so the label needs no edit when Boat is available again.",
  rationale: `${rationale} A provider that cannot take a job now stays in the order and is skipped.`,
  relatedDocs: docs,
  address,
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="boat-unavailable" />
  ),
});
