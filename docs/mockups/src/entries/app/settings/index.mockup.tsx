import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../components/paths.ts";
import { relatedMvp } from "../../../components/related-docs.ts";
import { orgAddress } from "../../../fixtures/org.ts";
import { LabelsScreen } from "../../../screens/settings/labels.tsx";

export const settings = defineScreen({
  title: "Settings",
  description:
    "Labels and their sizes, the first settings tab, on 18 November. mokly-ai uses three labels; the default ciproxy label starts a VM of 2 vCPU and 8 GB with 20 GB of free disk. Where jobs run says where the jobs of each label run now. The Members tab counts one open request for access.",
  rationale:
    "Only an Admin sees Settings, and the count on the Members tab tells the Admins of each request for access, because ciproxy sends no email. The size shows the free disk of each job, as the answer to item 77 says, and the caption says that a running job also holds disk for the image.",
  relatedDocs: relatedMvp("labels", "jobRouting", "security"),
  address: orgAddress("settings/labels"),
  useCasePaths: [paths.flowMemberAccess],
  render: (_props, context) => (
    <LabelsScreen viewport={context.viewport} state="list" />
  ),
});
