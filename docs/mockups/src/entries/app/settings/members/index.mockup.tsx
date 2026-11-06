import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { related, relatedMvp } from "../../../../components/related-docs.ts";
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
    "ciproxy cannot list the members of the GitHub organization without the Members permission, so a member asks for access by signing in, and the request shows here. A role needs current membership of one of the GitHub organizations of the workspace. Only Admins see Settings.",
  relatedDocs: related("security"),
  address: orgAddress("settings/members"),
  useCasePaths: [paths.flowGrantAdmin],
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
    {
      slug: "grant-admin",
      title: "Give the Admin role",
      description:
        "Cal Moore picks Admin for Dana Kim. Before her role changes, the dialog says that an Admin can add machines and connect Boat, which run the jobs of mokly-ai and mokly-labs with their secrets. It also says that an organization whose runners other people should control gets a workspace of its own.",
      rationale:
        "The answer to item 50: the Admin role alone adds a machine, connects Boat, and adds an organization, and ciproxy checks no GitHub right. So the grant of the role is the one explicit step of trust, and its dialog names it.",
      relatedDocs: relatedMvp("security", "workspaces"),
      props: { state: "grant-admin" },
      useCasePaths: [paths.flowGrantAdmin],
    },
  ],
});
