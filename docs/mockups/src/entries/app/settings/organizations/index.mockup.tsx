import { defineScreen } from "@mokly/mokly";

import { relatedMvp } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { OrganizationsScreen } from "../../../../screens/settings/organizations.tsx";

export const organizations = defineScreen({
  title: "Organizations",
  description:
    "The GitHub organizations of the workspace mokly-ai on 18 November: mokly-ai, which installed ciproxy on 1 September and gave the workspace its name, and mokly-labs, which an Admin added on 2 November. Each row shows the repository access of the organization and leads to its repositories. Add an organization opens GitHub, which decides who may finish the install. Take out takes an organization out of the workspace.",
  rationale:
    "The answer to item 48: a workspace is the customer, and it holds one or more GitHub organizations. It owns the tier, the members and their roles, the machines, the labels, the settings, and the Boat account, so a machine runs the jobs of every organization, and starts and machines count for the workspace.",
  relatedDocs: relatedMvp("security", "billing", "metering"),
  address: orgAddress("settings/organizations"),
  render: (_props, context) => (
    <OrganizationsScreen viewport={context.viewport} state="team" />
  ),
});
