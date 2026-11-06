import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../components/paths.ts";
import { related } from "../../../components/related-docs.ts";
import { orgAddress } from "../../../fixtures/org.ts";
import { LabelsScreen } from "../../../screens/settings/labels.tsx";

export const settings = defineScreen({
  title: "Settings",
  description:
    "Labels and their sizes, the first settings tab, on 18 November. mokly-ai uses three labels; the default ciproxy label starts a VM of 2 vCPU and 8 GB. Where jobs run says where the jobs of each label run now. The Members tab counts one open request for access.",
  rationale:
    "Only an Admin sees Settings, and the count on the Members tab tells the Admins of each request for access, because ciproxy sends no email.",
  relatedDocs: related("labels", "jobRouting", "security"),
  address: orgAddress("settings/labels"),
  useCasePaths: [paths.flowMemberAccess],
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="list" />
  ),
});
