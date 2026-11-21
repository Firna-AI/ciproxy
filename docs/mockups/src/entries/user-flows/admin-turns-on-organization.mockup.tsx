import { defineUseCase } from "@mokly/mokly";

import { paths } from "../../components/paths.ts";
import { relatedMvp } from "../../components/related-docs.ts";

export const adminTurnsOnOrganization = defineUseCase({
  title: "An Admin turns on an organization",
  description:
    "A what-if of December: Ana Souza adds mokly-studio on 8 December, while mokly-ai is on Free. After the check of the install, mokly-ai, her one Admin workspace, becomes its workspace. Her Admin role ends before ciproxy finishes the link, so mokly-studio joins inactive that day, for the plan and for an Admin. The move to Team on 11 December clears only the plan hold, and Cal Moore, a current Admin, turns it on.",
  relatedDocs: relatedMvp("workspaces", "billing", "security"),
  steps: [
    {
      screenPath: paths.inactiveBoth,
      title: "Inactive for two reasons",
      description:
        "Free includes one organization, and Ana is no longer an Admin. The row and the banner offer Choose Team first.",
    },
    {
      screenPath: paths.plansInactive,
      title: "Compare the plans",
      description: "Cal chooses Team and pays on Stripe.",
    },
    {
      screenPath: paths.inactiveAdmin,
      title: "Waiting for an Admin",
      description:
        "Team cleared the plan hold. mokly-studio still waits for an Admin.",
    },
    {
      screenPath: paths.inactiveTurnOn,
      title: "Turn on",
      description:
        "The dialog asks first, and it names the person who added mokly-studio.",
    },
    {
      screenPath: paths.inactiveAfterTeam,
      title: "Three organizations on Team",
      description: "mokly-studio runs its jobs, as mokly-ai and mokly-labs do.",
    },
  ],
});
