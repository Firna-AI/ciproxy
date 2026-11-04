import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../components/paths.ts";
import { planDoc, stateSchema } from "../../../components/state-schema.ts";
import { orgAddress } from "../../../fixtures/org.ts";
import { NoRoleScreen, noRoleStates } from "../../../screens/auth/no-role.tsx";

export const access = defineScreen({
  title: "No role yet",
  description:
    "Lena Fischer is a member of mokly-ai on GitHub with no role in ciproxy, so every page of mokly-ai shows only this one. Her sign-in asked the Admins for access, and the request shows in Settings, under Members. The page names the Admins.",
  rationale:
    "ciproxy controls access with its own roles, and it cannot list the members of the GitHub organization, so a sign-in is the request for access.",
  relatedDocs: [planDoc],
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
        "Tom Becker is an owner of mokly-ai on GitHub with no role in ciproxy. An owner can always claim the Admin role, so an organization never loses control of ciproxy. ciproxy checks the ownership with his own GitHub sign-in.",
      props: { state: "owner" },
    },
  ],
});
