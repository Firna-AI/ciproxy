import { defineUseCase } from "@mokly/mokly";

import { paths } from "../../components/paths.ts";
import { relatedMvp } from "../../components/related-docs.ts";

export const adminAddsOrganization = defineUseCase({
  title: "An Admin adds a second organization",
  description:
    "mokly-ai runs on Team with one organization. On 2 November, Cal Moore adds mokly-labs, so the workspace holds two organizations that share its plan, members, machines, labels, and Boat account.",
  relatedDocs: relatedMvp("workspaces", "security"),
  steps: [
    {
      screenPath: paths.organizationsOne,
      title: "One organization on Team",
      description:
        "mokly-ai is the last organization of the workspace, so it has no Take out. Cal chooses Add an organization, which opens the install of the App on GitHub.",
    },
    {
      screenPath: paths.organizations,
      title: "Two organizations",
      description:
        "GitHub installed the App on mokly-labs, so it joined the workspace of Cal. Its jobs use the machines, the labels, and the Boat account at once, and each organization now has Take out.",
    },
  ],
});
