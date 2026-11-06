import { defineScreen } from "@mokly/mokly";

import { relatedMvp } from "../../../components/related-docs.ts";
import { orgAddress } from "../../../fixtures/org.ts";
import { JobsLiveScreen } from "../../../screens/jobs/jobs-live.tsx";

export const workspaceMenu = defineScreen({
  slug: "workspace-menu",
  title: "Switch workspaces",
  description:
    "The switch of the top bar opens the workspaces of Cal Moore: mokly-ai on Team with its two organizations, and tidepool on Free, where he is a Viewer. Install on another organization makes a new workspace.",
  rationale:
    "The answer to item 48: the switch of the top bar switches workspaces, not organizations, because a workspace holds the jobs, the machines, and the plan of all its organizations.",
  relatedDocs: relatedMvp("workspaces", "security"),
  address: orgAddress("jobs"),
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="steady" workspaceMenu />
  ),
});
