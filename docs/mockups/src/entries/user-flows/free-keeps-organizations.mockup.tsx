import { defineUseCase } from "@mokly/mokly";

import { paths } from "../../components/paths.ts";
import { relatedMvp } from "../../components/related-docs.ts";

export const freeKeepsOrganizations = defineUseCase({
  title: "A workspace with two organizations moves to Free",
  description:
    "mokly-ai holds the organizations mokly-ai and mokly-labs on Team. In November, Cal Moore chooses Move to Free. Team stays until 3 December, and from 4 December mokly-ai is on Free with both organizations, whose jobs keep running.",
  relatedDocs: relatedMvp("workspaces", "billing", "metering"),
  steps: [
    {
      screenPath: paths.moveToFree,
      title: "Move to Free",
      description:
        "A workspace with more than one organization can choose Move to Free. The dialog asks first.",
    },
    {
      screenPath: paths.freeScheduled,
      title: "Team ends at the end of the period",
      description:
        "Team and its add-ons stay until 3 December. Until then, Keep Team undoes the move.",
    },
    {
      screenPath: paths.freeOrganizations,
      title: "On Free with two organizations",
      description:
        "Free includes one organization, so the banner says that mokly-ai passed a limit of its plan and offers Team. The pace of the starts passes Free on 9 December.",
    },
    {
      screenPath: paths.organizationsFreeOver,
      title: "The same banner on Settings",
      description:
        "Both organizations stay in the workspace, and each of the two keeps its Take out. A new organization would join inactive until Team.",
    },
  ],
});
