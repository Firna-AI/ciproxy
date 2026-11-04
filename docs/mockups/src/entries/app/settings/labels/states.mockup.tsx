import { defineScreen } from "@mokly/mokly";

import { planDoc } from "../../../../components/state-schema.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { LabelsScreen } from "../../../../screens/settings/labels.tsx";

const address = orgAddress("settings/labels");

export const editLabel = defineScreen({
  slug: "edit-label",
  title: "Edit a label",
  description:
    "The dialog that edits the size of ciproxy-8vcpu, where its jobs may run, and what they do when nothing fits. It says how many jobs of the size fit on one machine.",
  relatedDocs: [planDoc],
  address,
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="edit-label" />
  ),
});

export const noMachineFits = defineScreen({
  slug: "no-machine-fits",
  title: "A size that fits no machine",
  description:
    "A new label with 16 vCPU and 32 GB. An AX42 offers 12 vCPU to jobs, so no own machine fits it, and the dialog says before the save that its jobs run on Boat. The largest Boat size, 16 vCPU and 32 GB, fits.",
  relatedDocs: [planDoc],
  address,
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="no-machine-fits" />
  ),
});

export const nothingFits = defineScreen({
  slug: "nothing-fits",
  title: "A size that nothing fits",
  description:
    "ciproxy-32vcpu asks for 32 vCPU and 128 GB, more than any own machine and the largest Boat size. Its setting When nothing fits decides: Wait, the default, keeps its jobs queued; Run on the largest Boat size starts them on 16 vCPU and 32 GB.",
  rationale:
    "A label can stop fitting after a machine leaves or the Boat plan changes, so ciproxy never rejects such a label on save.",
  relatedDocs: [planDoc],
  address,
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="nothing-fits" />
  ),
});

export const labelWaits = defineScreen({
  slug: "label-waits",
  title: "A label whose jobs wait",
  description:
    "The list after mokly-ai added ciproxy-32vcpu for load tests. Where jobs run says that its jobs wait, because nothing fits; the live queue says the same.",
  relatedDocs: [planDoc],
  address,
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="label-waits" />
  ),
});
