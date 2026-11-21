import { defineScreen } from "@mokly/mokly";

import { relatedMvp } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { LabelsScreen } from "../../../../screens/settings/labels.tsx";

export const refusedName = defineScreen({
  slug: "refused-name",
  title: "A refused label name",
  description:
    "Cal Moore adds a label for load tests and names it load tests. A label name follows the GitHub label rules, which allow no space, so the field says so and offers load-tests. Add label waits for a name that the rules allow. Before the error, the field says that the name cannot change later.",
  rationale:
    "Design call 33: label names follow the GitHub label rules, and a label has no rename. The space is one sample of the rules, which the docs list.",
  relatedDocs: relatedMvp("labels", "jobRouting"),
  address: orgAddress("settings/labels"),
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="refused-name" />
  ),
});
