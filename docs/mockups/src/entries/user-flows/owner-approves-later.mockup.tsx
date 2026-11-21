import { defineUseCase } from "@mokly/mokly";

import { paths } from "../../components/paths.ts";
import { relatedMvp } from "../../components/related-docs.ts";

export const ownerApprovesLater = defineUseCase({
  title: "An owner approves a request later",
  description:
    "A what-if of 2 November: Cal Moore is no owner of mokly-labs, so GitHub sends his request to its owners. An owner approves it later that day, while no ciproxy page is open. The install then waits for Cal, the only person who can choose its workspace, until his next sign-in. As in the picker what-if, he is also an Admin of driftwood.",
  relatedDocs: relatedMvp("workspaces", "security"),
  steps: [
    {
      screenPath: paths.organizationsOne,
      title: "One organization on Team",
      description:
        "Cal chooses Add an organization and asks for the install on GitHub.",
    },
    {
      screenPath: paths.addRequestWait,
      title: "Waiting for an owner",
      description:
        "The return names the organization of the request and waits. ciproxy picks no workspace before the approval and the check of the install.",
    },
    {
      screenPath: paths.addApproved,
      title: "Choose a workspace",
      description:
        "At his next sign-in, ciproxy checks the install with his own GitHub sign-in and lists his Admin workspaces. Cal picks mokly-ai.",
    },
    {
      screenPath: paths.organizations,
      title: "Two organizations",
      description:
        "mokly-labs joined mokly-ai, and its jobs use the machines, the labels, and the Boat account at once.",
    },
  ],
});
