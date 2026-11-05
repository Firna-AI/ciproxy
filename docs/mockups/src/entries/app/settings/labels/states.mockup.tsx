import { defineScreen } from "@mokly/mokly";

import { related, relatedMvp } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { LabelsScreen } from "../../../../screens/settings/labels.tsx";

const address = orgAddress("settings/labels");

export const editLabel = defineScreen({
  slug: "edit-label",
  title: "Edit a label",
  description:
    "The dialog that edits the size of ciproxy-8vcpu, where its jobs may run, and what they do when nothing fits. It says how many jobs of the size fit on one machine. The name shows with a lock, because a label keeps its name; Delete label leads the footer, for a rename by a new label.",
  rationale:
    "Design call 33: label names follow the GitHub label rules, and a label has no rename, because a new label and a delete give a safe rename.",
  relatedDocs: relatedMvp("labels", "jobRouting", "overflow"),
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
  relatedDocs: related("labels", "jobRouting", "overflow"),
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
  relatedDocs: related("labels", "jobRouting", "overflow"),
  address,
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="nothing-fits" />
  ),
});

export const mixedMachines = defineScreen({
  slug: "mixed-machines",
  title: "Machines that fit different numbers of jobs",
  description:
    "The dialog of ciproxy-4vcpu after office-mini-1 joined as the fourth machine. An Admin raises the disk of the label from 80 to 100 GB, and before the save the dialog says what the new size gives. Each Hetzner AX42 runs 3 jobs of the size at a time. office-mini-1 has the slots for 3 too, but its disk holds only 2 jobs of 100 GB, so the dialog says 2 to 3 jobs. The banner offers the extra-machine add-on, because Team includes three machines.",
  rationale:
    "Design call 41: a size that fits a different number of jobs on different machines shows a range, which keeps the rule of one machine: a job runs on one machine.",
  relatedDocs: relatedMvp("labels", "jobRouting"),
  address,
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="mixed-machines" />
  ),
});

export const labelWaits = defineScreen({
  slug: "label-waits",
  title: "A label whose jobs wait",
  description:
    "The list after mokly-ai added ciproxy-32vcpu for load tests. Where jobs run says that its jobs wait, because nothing fits; the live queue says the same.",
  relatedDocs: related("labels", "jobRouting", "overflow"),
  address,
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="label-waits" />
  ),
});
