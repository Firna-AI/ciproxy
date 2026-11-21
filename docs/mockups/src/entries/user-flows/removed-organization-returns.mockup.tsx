import { defineUseCase } from "@mokly/mokly";

import { paths } from "../../components/paths.ts";
import { relatedMvp } from "../../components/related-docs.ts";

export const removedOrganizationReturns = defineUseCase({
  title: "A removed organization comes back",
  description:
    "A what-if of 20 November: an owner of mokly-labs removes the ciproxy GitHub App from mokly-labs on GitHub. mokly-ai keeps running its jobs on the machines and Boat of the workspace, and its subscription stays. Cal Moore installs the App on mokly-labs again before the deadline, so mokly-labs comes back to mokly-ai with its job history.",
  relatedDocs: relatedMvp("appRemoval", "workspaces"),
  steps: [
    {
      screenPath: paths.organizationsRemoved,
      title: "The App was removed",
      description:
        "mokly-labs shows as removed, with the day before which a new install keeps its job history. mokly-ai is the last linked organization, so it has no Take out.",
    },
    {
      screenPath: paths.organizations,
      title: "Installed again",
      description:
        "Install again opens GitHub. Cal is an Admin of one workspace, so the return links mokly-labs to mokly-ai at once, with its job history.",
    },
  ],
});
