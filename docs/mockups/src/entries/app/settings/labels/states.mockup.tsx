import { defineScreen } from "@mokly/mokly";

import { related, relatedMvp } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { LabelsScreen } from "../../../../screens/settings/labels.tsx";

const address = orgAddress("settings/labels");

export const editLabel = defineScreen({
  slug: "edit-label",
  title: "Edit a label",
  description:
    "The dialog that edits the size of ciproxy-8vcpu, where its jobs may run, and what they do when nothing fits. Its disk is the free disk of each job, 60 GB, with the least value, 1 GB, as the hint of the field. The line under the size gives the VM disk with the current image, 123 GB. It says how many jobs of the size fit on one machine. The name shows with a lock, because a label keeps its name; Delete label leads the footer, for a rename by a new label.",
  rationale:
    "Design call 33: label names follow the GitHub label rules, and a label has no rename, because a new label and a delete give a safe rename. The answer to item 77: the disk of a label is the free disk of each job, as GitHub states the disk of its runners, and the VM disk adds the image.",
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
    "A new label with 16 vCPU, 32 GB, and 120 GB of free disk. An AX42 offers 12 vCPU to jobs, so no own machine fits it, and the dialog says before the save that its jobs run on Boat. The largest Boat size, xlarge with 16 vCPU and 32 GB, fits, because its 251 GB disk holds the VM disk of 185 GB.",
  relatedDocs: relatedMvp("labels", "jobRouting", "overflow"),
  address,
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="no-machine-fits" />
  ),
});

export const nothingFits = defineScreen({
  slug: "nothing-fits",
  title: "A size that nothing fits",
  description:
    "ciproxy-32vcpu asks for 32 vCPU, 128 GB, and 200 GB of free disk, more than any own machine and the largest Boat size. Its setting When nothing fits decides: Wait, the default, keeps its jobs queued; Run on the largest Boat size starts them on 16 vCPU and 32 GB. The 251 GB disk of that size holds the image but gives a job less free disk than the label.",
  rationale:
    "A label can stop fitting after a machine leaves or the Boat plan changes, so ciproxy never rejects such a label on save. The answer to item 77: the largest Boat size can give less free disk than the label, as it gives less vCPU and memory, but never a size that cannot hold the image.",
  relatedDocs: relatedMvp("labels", "jobRouting", "overflow"),
  address,
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="nothing-fits" />
  ),
});

export const mixedMachines = defineScreen({
  slug: "mixed-machines",
  title: "Machines that fit different numbers of jobs",
  description:
    "The dialog of ciproxy-4vcpu after office-mini-1 joined as the fourth machine. An Admin raises the free disk of the label from 20 to 40 GB, and before the save the dialog says what the new size gives: a VM disk of 102 GB with the current image. Each Hetzner AX42 runs 3 jobs of the size at a time. office-mini-1 has the slots for 3 too, but its 279 GB of job disk holds only 2 such VM disks, so the dialog says 2 to 3 jobs. The banner offers the extra-machine add-on, because Team includes three machines.",
  rationale:
    "Design call 41: a size that fits a different number of jobs on different machines shows a range, which keeps the rule of one machine: a job runs on one machine. The answer to item 77: the fit counts the whole VM disk of a job, the image with its free disk.",
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
