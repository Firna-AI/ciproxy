import { defineUseCase } from "@mokly/mokly";

import { paths } from "../../components/paths.ts";
import { relatedMvp } from "../../components/related-docs.ts";

export const adminGivesAdminRole = defineUseCase({
  title: "An Admin gives the Admin role",
  description:
    "Cal Moore gives Dana Kim, who has the Billing role, the Admin role on 18 November. Before her role changes, a dialog says what the Admin role controls in the workspace mokly-ai.",
  relatedDocs: relatedMvp("security", "workspaces"),
  steps: [
    {
      screenPath: paths.members,
      title: "Members and roles",
      description:
        "Cal opens Settings, under Members, and picks Admin in the role picker of Dana Kim.",
    },
    {
      screenPath: paths.grantAdmin,
      title: "What the Admin role controls",
      description:
        "The dialog says that an Admin can add machines and connect Boat, which run the jobs of mokly-ai and mokly-labs with their secrets, and that an organization whose runners other people should control gets a workspace of its own. Cal gives the role.",
    },
  ],
});
