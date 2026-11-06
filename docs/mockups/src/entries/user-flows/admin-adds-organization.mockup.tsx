import { defineUseCase } from "@mokly/mokly";

import { paths } from "../../components/paths.ts";
import { relatedMvp } from "../../components/related-docs.ts";

export const adminAddsOrganization = defineUseCase({
  title: "An Admin adds a second organization",
  description:
    "mokly-ai runs on Team with one organization. On 2 November, Cal Moore adds mokly-labs. GitHub installs the App on it and returns to ciproxy, which checks the install with his own GitHub sign-in. Cal is an Admin of one workspace, so mokly-labs joins mokly-ai at once, and the workspace holds two organizations that share its plan, members, machines, labels, and Boat account.",
  relatedDocs: relatedMvp("workspaces", "security"),
  steps: [
    {
      screenPath: paths.organizationsOne,
      title: "One organization on Team",
      description:
        "mokly-ai is the last organization of the workspace, so it has no Take out. Cal chooses Add an organization, which opens the install of the App on GitHub, where he picks mokly-labs.",
    },
    {
      screenPath: paths.organizations,
      title: "Two organizations",
      description:
        "ciproxy checked the install with Cal's own GitHub sign-in, and he is an Admin of one workspace, so the return linked mokly-labs to it with no page between. Its jobs use the machines, the labels, and the Boat account at once, and each organization now has Take out.",
    },
  ],
});
