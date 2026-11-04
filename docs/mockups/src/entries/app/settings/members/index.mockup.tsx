import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { related } from "../../../../components/related-docs.ts";
import { stateSchema } from "../../../../components/state-schema.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import {
  MembersScreen,
  membersStates,
} from "../../../../screens/settings/members.tsx";

export const members = defineScreen({
  title: "Members and roles",
  description:
    "The people of mokly-ai in ciproxy and their roles: Admin, Billing, and Viewer. Cal Moore installed the GitHub App, so he is the first Admin. An Admin changes a role with its picker, or removes a person.",
  rationale:
    "ciproxy cannot list the members of the GitHub organization without the Members permission, so a member asks for access by signing in, and the request shows here. A role needs current membership of the GitHub organization. Only Admins see Settings.",
  relatedDocs: related("security"),
  address: orgAddress("settings/members"),
  propSchema: stateSchema(membersStates),
  props: { state: "members" },
  render: (props, context) => (
    <MembersScreen viewport={context.viewport} state={props.state} />
  ),
  variants: [
    {
      slug: "request",
      title: "A member asks for access",
      description:
        "Lena Fischer, a member of mokly-ai on GitHub with no role, signed in on 18 November, which asked for access. The Members tab counts the open request on every settings page. An Admin picks her role and gives access, or declines.",
      props: { state: "request" },
      useCasePaths: [paths.flowMemberAccess],
    },
  ],
});
