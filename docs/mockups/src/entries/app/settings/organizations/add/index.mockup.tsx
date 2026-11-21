import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../../components/paths.ts";
import { relatedMvp } from "../../../../../components/related-docs.ts";
import { stateSchema } from "../../../../../components/state-schema.ts";
import { host } from "../../../../../fixtures/org.ts";
import {
  ChooseWorkspaceScreen,
  chooseWorkspaceStates,
} from "../../../../../screens/settings/choose-workspace.tsx";

export const addOrganization = defineScreen({
  title: "Add an organization",
  description:
    "The return from GitHub after Add an organization, in a what-if of 2 November: Cal Moore is also an Admin of driftwood, a workspace on Free. On GitHub, he picked mokly-labs. ciproxy checked the install with his own GitHub sign-in, and then it lists the workspaces where he is an Admin, in the order of the workspace switch, with none picked. tidepool, where he is a Viewer, is not in the list. On driftwood, mokly-labs would join inactive, because Free includes one organization. The button names the organization, and after the pick also the workspace. With one Admin workspace, the return links at once and does not show this page.",
  rationale:
    "The answer B to item 82: GitHub keeps the choice of the organization, and the person picks the workspace, because ciproxy cannot know it. The workspace where Add started is no pick, so ciproxy never guesses one. The list offers no new workspace: an install that no Admin started with Add an organization makes one, as before.",
  relatedDocs: relatedMvp("workspaces", "security", "billing"),
  address: `${host}/add-organization`,
  useCasePaths: [paths.flowChooseWorkspace],
  propSchema: stateSchema(chooseWorkspaceStates),
  props: { state: "returned" },
  render: (props, context) => (
    <ChooseWorkspaceScreen viewport={context.viewport} state={props.state} />
  ),
  variants: [
    {
      slug: "approved",
      title: "Choose a workspace after a late approval",
      description:
        "Cal Moore is no owner of mokly-labs, so his request waited for an owner, who approved it later on 2 November while no ciproxy page was open. The install waits for Cal alone. At his next sign-in, ciproxy checks the install with his own GitHub sign-in and lists his Admin workspaces. He picked mokly-ai, so the button reads Add mokly-labs to mokly-ai.",
      rationale:
        "The answer B to item 82: a late approval waits for the person who asked to sign in again and choose. The approval itself picks no workspace.",
      props: { state: "approved" },
      useCasePaths: [paths.flowApprovedLater],
    },
    {
      slug: "request-wait",
      title: "Waiting for an owner",
      description:
        "Cal Moore is no owner of mokly-labs, so GitHub sends his request to its owners and returns. The request names the organization, but no install exists yet, so ciproxy picks no workspace. Check again continues once an owner approves. Back to Organizations opens the tab where Cal started, which stays as it was.",
      props: { state: "request-wait" },
      useCasePaths: [paths.flowApprovedLater],
    },
    {
      slug: "not-confirmed",
      title: "The install is not confirmed",
      description:
        "ciproxy could not confirm the install with Cal's own GitHub sign-in, for example because that sign-in did not finish. The page names the failure and offers one action: a new GitHub sign-in, which checks the install again. No workspace gets the organization before the check passes.",
      props: { state: "not-confirmed" },
    },
    {
      slug: "no-admin-workspace",
      title: "No workspace where the person is an Admin",
      description:
        "Ana Souza asked the owners of mokly-labs for the install. When an owner approved it, she was an Admin of no workspace. The install waits for her, and ciproxy shows it to no other person and links it to no workspace. Once an Admin gives her the Admin role, Check again links it at once, or lists her Admin workspaces when she has several.",
      props: { state: "no-admin-workspace" },
    },
  ],
});
