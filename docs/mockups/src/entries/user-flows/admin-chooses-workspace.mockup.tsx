import { defineUseCase } from "@mokly/mokly";

import { paths } from "../../components/paths.ts";
import { relatedMvp } from "../../components/related-docs.ts";

export const adminChoosesWorkspace = defineUseCase({
  title: "An Admin of two workspaces picks one",
  description:
    "A what-if of 2 November: Cal Moore is also an Admin of driftwood, a workspace on Free. He adds mokly-labs from mokly-ai. After the check of the install, the return lists both workspaces with none picked, and Cal picks the one that runs the jobs of mokly-labs.",
  relatedDocs: relatedMvp("workspaces", "security", "billing"),
  steps: [
    {
      screenPath: paths.organizationsOne,
      title: "One organization on Team",
      description:
        "Cal chooses Add an organization and installs the App on mokly-labs on GitHub.",
    },
    {
      screenPath: paths.addOrganization,
      title: "Choose a workspace",
      description:
        "ciproxy checked the install with Cal's own GitHub sign-in. The return lists mokly-ai and driftwood with none picked. On driftwood, mokly-labs would join inactive until Team. Cal picks mokly-ai, and the button names both: Add mokly-labs to mokly-ai.",
    },
    {
      screenPath: paths.organizations,
      title: "Two organizations",
      description:
        "mokly-labs joined mokly-ai, and its jobs use the machines, the labels, and the Boat account at once.",
    },
  ],
});
