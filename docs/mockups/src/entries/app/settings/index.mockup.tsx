import { defineScreen } from "@mokly/mokly";

import { planDoc } from "../../../components/state-schema.ts";
import { orgAddress } from "../../../fixtures/org.ts";
import { LabelsScreen } from "../../../screens/settings/labels.tsx";

export const settings = defineScreen({
  title: "Settings",
  description:
    "Labels and machine shapes, the first settings tab. mokly-ai uses three labels; the default ciproxy label starts a 2 vCPU, 8 GB VM.",
  relatedDocs: [planDoc],
  address: orgAddress("settings/labels"),
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="list" />
  ),
});
