import { defineScreen } from "@mokly/mokly";

import { planDoc } from "../../../../components/state-schema.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { LabelsScreen } from "../../../../screens/settings/labels.tsx";

const address = orgAddress("settings/labels");

export const editShape = defineScreen({
  slug: "edit-shape",
  title: "Edit a label",
  description:
    "The dialog that edits the shape of ciproxy-8vcpu and where its jobs may run. It says how many jobs of the shape fit on one machine.",
  relatedDocs: [planDoc],
  address,
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="edit-shape" />
  ),
});

export const noMachineFits = defineScreen({
  slug: "no-machine-fits",
  title: "A shape that fits no machine",
  description:
    "A new label with 32 vCPU. No own machine offers that much, so its jobs run on Boat only, and the dialog says so before saving.",
  relatedDocs: [planDoc],
  address,
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="no-machine-fits" />
  ),
});
