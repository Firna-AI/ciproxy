import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { relatedMvp } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { OrganizationsScreen } from "../../../../screens/settings/organizations.tsx";

export const organizations = defineScreen({
  title: "Organizations",
  description:
    "The workspace mokly-ai and its GitHub organizations on 18 November: mokly-ai, which installed ciproxy on 1 September and gave the workspace its name, and mokly-labs, which an Admin added on 2 November. Rename changes the name of the workspace. Each row shows the repository access of the organization and leads to its repositories. Add an organization opens GitHub, which decides who may finish the install. On the return, ciproxy checks the install with the person's own GitHub sign-in. With one workspace where the person is an Admin, the organization joins it at once. With several, the return asks which. Take out takes an organization out of the workspace.",
  rationale:
    "The answer to item 48: a workspace is the customer, and it holds one or more GitHub organizations. It owns the tier, the members and their roles, the machines, the labels, the settings, and the Boat account, so a machine runs the jobs of every organization, and starts and machines count for the workspace.",
  relatedDocs: relatedMvp("workspaces", "security", "billing", "metering"),
  address: orgAddress("settings/organizations"),
  useCasePaths: [
    paths.flowAddOrganization,
    paths.flowChooseWorkspace,
    paths.flowApprovedLater,
  ],
  render: (_props, context) => (
    <OrganizationsScreen viewport={context.viewport} state="team" />
  ),
});
