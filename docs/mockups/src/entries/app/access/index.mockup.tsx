import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../components/paths.ts";
import { related, relatedMvp } from "../../../components/related-docs.ts";
import { stateSchema } from "../../../components/state-schema.ts";
import { orgAddress } from "../../../fixtures/org.ts";
import { NoRoleScreen, noRoleStates } from "../../../screens/auth/no-role.tsx";

export const access = defineScreen({
  title: "No role yet",
  description:
    "Lena Fischer is a member of mokly-ai on GitHub with no role in ciproxy, so every page of mokly-ai shows only this one. Her sign-in asked the Admins for access, and the request shows in Settings, under Members. The page names the Admins.",
  rationale:
    "ciproxy controls access with its own roles, and the person's own sign-in proves membership without a full member list of the GitHub organization, so a sign-in is the request for access.",
  relatedDocs: related("security"),
  address: orgAddress("jobs"),
  useCasePaths: [paths.flowMemberAccess],
  propSchema: stateSchema(noRoleStates),
  props: { state: "member" },
  render: (props, context) => (
    <NoRoleScreen viewport={context.viewport} state={props.state} />
  ),
  variants: [
    {
      slug: "owner",
      title: "An owner claims the Admin role",
      description:
        "Tom Becker owns both organizations of the workspace, mokly-ai and mokly-labs, on GitHub, and has no role in ciproxy. ciproxy offers him the Admin claim, so the page offers Claim the Admin role beside the Admins. An owner of only some organizations sees the member page while the workspace has an Admin.",
      rationale:
        "An owner claims the Admin role when the workspace has no Admin, or when the owner owns every organization of the workspace, so no organization takes over another. ciproxy checks the ownership with his own GitHub sign-in, and the page reads only the claim that ciproxy offers, as the dashboard reads the claim of the session.",
      props: { state: "owner" },
    },
    {
      slug: "left",
      title: "A person who left the organization",
      description:
        "Jonas Weber, a Viewer, left mokly-ai on GitHub, and he is no member of mokly-labs either. The membership check of the next hour ended his role, so every page of the workspace opens this page. He sees no data of the workspace. His sign-in has no workspace now, so Check again needs a new GitHub sign-in and opens the terms step first. When an owner adds him again to one of its organizations, that sign-in leads to the page that asks the Admins for access.",
      rationale:
        "Design call 35: ciproxy checks the membership of each person each hour, so access ends soon after a person leaves the organization. A role needs current membership, and ciproxy keeps no record of a person without a role or a request.",
      relatedDocs: relatedMvp("security"),
      props: { state: "left" },
    },
  ],
});
