import { defineScreen } from "@mokly/mokly";

import { planDoc } from "../../../components/state-schema.ts";
import { orgAddress } from "../../../fixtures/org.ts";
import { LabelsScreen } from "../../../screens/settings/labels.tsx";

export const settings = defineScreen({
  title: "Settings",
  description:
    "Labels and their sizes, the first settings tab. mokly-ai uses three labels; the default ciproxy label starts a VM of 2 vCPU and 8 GB. Where jobs run says where the jobs of each label run now.",
  relatedDocs: [planDoc],
  address: orgAddress("settings/labels"),
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="list" />
  ),
});
