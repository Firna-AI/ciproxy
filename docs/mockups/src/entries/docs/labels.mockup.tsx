import { defineScreen } from "@mokly/mokly";

import { relatedMvp } from "../../components/related-docs.ts";
import { siteAddress } from "../../fixtures/site.ts";
import { LabelsReferenceScreen } from "../../screens/docs/labels-reference.tsx";

export const labelsReference = defineScreen({
  title: "Labels and VM sizes",
  description:
    "The reference page of labels: the default Linux and macOS labels with their platforms, sizes, and slots, the section macOS labels, the free disk of a label and the VM disk of a job, how slots work, custom labels with the setting When nothing fits, and where a job runs.",
  rationale:
    "The labels and sizes are the ones that Settings shows for mokly-ai, so a reader finds the same values in the dashboard. As the answer to item 77 says, the disk of a label is the free disk of each job, as GitHub states the disk of its runners. The labels doc says that a label keeps its platform, and that a macOS label runs on the Macs only. The sizes of the macOS labels are placeholders until the protocol docs set them.",
  relatedDocs: relatedMvp("labels", "jobRouting", "overflow", "overflowSpend"),
  address: siteAddress("docs/reference/labels"),
  render: (_props, context) => (
    <LabelsReferenceScreen viewport={context.viewport} />
  ),
});
