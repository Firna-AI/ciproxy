import { defineUseCase } from "@mokly/mokly";

import { paths } from "../../components/paths.ts";
import { relatedMvp } from "../../components/related-docs.ts";

export const freeAddsOrganization = defineUseCase({
  title: "A Free workspace adds an organization",
  description:
    "mokly-ai is on Free from 4 December and keeps mokly-ai and mokly-labs. On 8 December, Cal Moore adds mokly-studio. Free includes one organization, so mokly-studio joins inactive, while the other two keep running. The move to Team turns it on with no other step.",
  relatedDocs: relatedMvp("workspaces", "billing"),
  steps: [
    {
      screenPath: paths.organizationsFreeOver,
      title: "On Free with two organizations",
      description:
        "Free still offers Add an organization, and the caption says that a new one stays inactive until Team. Cal adds mokly-studio on GitHub.",
    },
    {
      screenPath: paths.inactivePlan,
      title: "mokly-studio is inactive",
      description:
        "Cal is an Admin of one workspace, so mokly-studio joined mokly-ai at once, inactive. Its row says why and offers Choose Team, and the banner says that jobs keep running in mokly-ai and mokly-labs.",
    },
    {
      screenPath: paths.plansInactive,
      title: "Compare the plans",
      description: "Cal chooses Team and pays on Stripe.",
    },
    {
      screenPath: paths.inactiveAfterTeam,
      title: "Three organizations on Team",
      description: "On Team, mokly-studio runs its jobs at once.",
    },
  ],
});
