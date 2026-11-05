import { defineScreen } from "@mokly/mokly";

import { relatedMvp } from "../../../../../components/related-docs.ts";
import { orgAddress } from "../../../../../fixtures/org.ts";
import { OrganizationsScreen } from "../../../../../screens/settings/organizations.tsx";

export const renameWorkspace = defineScreen({
  title: "Rename the workspace",
  description:
    "Cal Moore renames the workspace mokly-ai to Mokly from the Organizations tab. The name shows in the workspace switch, and the GitHub organizations keep their names.",
  rationale:
    "A design call of the docs work of items 32 and 48: a new workspace takes the name of its first organization, and an Admin can rename it.",
  relatedDocs: relatedMvp("workspaces"),
  address: orgAddress("settings/organizations"),
  render: (_props, context) => (
    <OrganizationsScreen viewport={context.viewport} state="rename" />
  ),
});
