import { defineScreen } from "@mokly/mokly";

import { planDoc } from "../../components/state-schema.ts";
import { siteAddress } from "../../fixtures/site.ts";
import { LabelsReferenceScreen } from "../../screens/docs/labels-reference.tsx";

export const labelsReference = defineScreen({
  title: "Labels and machine sizes",
  description:
    "The reference page of labels: the default labels with their sizes and slots, how slots work, custom labels, and where a job runs.",
  rationale:
    "The labels and sizes are the ones that Settings shows for mokly-ai, so a reader finds the same values in the dashboard.",
  relatedDocs: [planDoc],
  address: siteAddress("docs/reference/labels"),
  render: (_props, context) => (
    <LabelsReferenceScreen viewport={context.viewport} />
  ),
});
