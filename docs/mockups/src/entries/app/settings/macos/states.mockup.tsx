import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { LabelsScreen } from "../../../../screens/settings/labels.tsx";

const address = orgAddress("settings/labels");

const rationale =
  "The labels doc says that a label has a platform, Linux or macOS, which never changes after the label is created, as its name does not. A macOS label runs only on the Macs and never on Boat, because a cloud Mac bills 24 hours at least. A Mac uses the slot rule of a Linux machine, and it runs at most 2 job VMs at a time, because Apple allows at most 2 macOS VMs at a time on one Mac. The sizes of the default macOS labels and the size of the macOS image are placeholders until the protocol docs and the image build set them.";

export const labels = defineScreen({
  slug: "labels",
  title: "Labels with a macOS label",
  description:
    "A what-if of 18 November: mokly-ai added a Mac, mac-studio-1, as its fourth machine, and runs jobs on the default macOS labels ciproxy-macos and ciproxy-macos-medium. Each label names its platform under its name. The jobs of a macOS label run on the Macs only, and the caption says that a Mac runs at most 2 job VMs at a time. Four machines pass the three of Team, so the banner offers the extra-machine add-on. Add label opens the add dialog with macOS picked, and Edit of ciproxy-macos opens its dialog.",
  rationale,
  relatedDocs: related("labels", "jobRouting", "overflow"),
  address,
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="macos" />
  ),
});

export const addLabel = defineScreen({
  slug: "add-label",
  title: "Add a macOS label",
  description:
    "The add dialog of ciproxy-macos-xcode, 8 vCPU and 32 GB for Xcode builds, with macOS picked as the platform. The hint says that the platform cannot change later. The VM disk uses the macOS image. The fit line counts the Macs only: a job of this size takes 4 slots, so mac-studio-1 runs 1 at a time. The dialog has no Boat switch and no choice of When nothing fits: the jobs wait for a free Mac.",
  rationale,
  relatedDocs: related("labels", "jobRouting", "overflow"),
  address,
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="add-macos" />
  ),
});

export const editLabel = defineScreen({
  slug: "edit-label",
  title: "Edit a macOS label",
  description:
    "The dialog of ciproxy-macos. The platform shows with a lock beside the name, and its hint says to add a new label to change it. The size, the fit line, and the Mac-only line stay as in the add dialog.",
  rationale,
  relatedDocs: related("labels", "jobRouting", "overflow"),
  address,
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="edit-macos" />
  ),
});
