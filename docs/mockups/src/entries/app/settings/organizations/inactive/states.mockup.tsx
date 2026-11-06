import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../../components/paths.ts";
import { relatedMvp } from "../../../../../components/related-docs.ts";
import { orgAddress } from "../../../../../fixtures/org.ts";
import { OrganizationsScreen } from "../../../../../screens/settings/organizations.tsx";

const address = orgAddress("settings/organizations");
const relatedDocs = relatedMvp("workspaces", "billing", "security");

export const inactivePlan = defineScreen({
  slug: "plan",
  title: "Inactive until Team",
  description:
    "On 8 December, mokly-ai is on Free and keeps mokly-ai and mokly-labs from Team. Cal Moore adds mokly-studio. He is an Admin of one workspace, so the return links it to mokly-ai at once. Free includes one organization, so mokly-studio joins inactive: it gets no runners, and its jobs wait on GitHub. Its row says why, offers Choose Team, and keeps Take out. The tab's banner says the same, and that jobs keep running in mokly-ai and mokly-labs. It takes the place of the soft-limit banner of the organizations, because Team clears both.",
  rationale:
    "The answer D to item 83: the organization joins the workspace, inactive until the plan allows it, and the other organizations keep running. Every linked organization counts, also an inactive one, and the organizations that a workspace kept on Free stay active.",
  relatedDocs,
  address,
  useCasePaths: [paths.flowFreeAddsOrganization],
  render: (_props, context) => (
    <OrganizationsScreen viewport={context.viewport} state="inactive-plan" />
  ),
});

export const bothHolds = defineScreen({
  slug: "both-holds",
  title: "Inactive for the plan and for an Admin",
  description:
    "A what-if of 8 December: Ana Souza adds mokly-studio. After the check of the install, mokly-ai, her one Admin workspace, becomes its workspace. Her Admin role ends before ciproxy finishes the link, so mokly-studio joins mokly-ai inactive that day with two holds: Free includes one organization, and the person who added it is no longer an Admin. The row and the banner name both reasons and offer Choose Team first. After the move to Team, the row and the banner offer Turn on.",
  rationale:
    "The workspace stays the one that became its workspace after the check of the install. The lost role moves the organization to no other workspace and gives no role to anyone. ciproxy records the holds when it finishes the link. Team clears only the plan hold, and a current Admin clears the other.",
  relatedDocs,
  address,
  useCasePaths: [paths.flowTurnOnOrganization],
  render: (_props, context) => (
    <OrganizationsScreen viewport={context.viewport} state="both-holds" />
  ),
});

export const inactiveAdmin = defineScreen({
  slug: "admin",
  title: "Waiting for an Admin",
  description:
    "On 11 December, mokly-ai moves to Team, which clears only the plan hold of mokly-studio. The hold for an Admin stays from 8 December, because Ana Souza, who added it, is no longer an Admin. The banner and the row offer Turn on to an Admin, and mokly-ai and mokly-labs keep running.",
  rationale:
    "A current Admin decides whether an organization that a former Admin added may use the machines, the labels, the settings, and the Boat account of the workspace.",
  relatedDocs,
  address,
  useCasePaths: [paths.flowTurnOnOrganization],
  render: (_props, context) => (
    <OrganizationsScreen viewport={context.viewport} state="inactive-admin" />
  ),
});

export const turnOn = defineScreen({
  slug: "turn-on",
  title: "Turn on an organization",
  description:
    "Cal Moore turns on mokly-studio. The dialog asks first, because its jobs then use the machines, the labels, the settings, and the Boat account of the workspace, and it names the person who added it.",
  relatedDocs,
  address,
  useCasePaths: [paths.flowTurnOnOrganization],
  render: (_props, context) => (
    <OrganizationsScreen viewport={context.viewport} state="turn-on" />
  ),
});

export const afterTeam = defineScreen({
  slug: "after-team",
  title: "Active after the move to Team",
  description:
    "mokly-ai on Team on 11 December with three organizations. After the move to Team, mokly-studio runs its jobs with no other step, as mokly-ai and mokly-labs do. In the what-if of Ana Souza, it runs once Cal turns it on.",
  relatedDocs,
  address,
  useCasePaths: [paths.flowFreeAddsOrganization, paths.flowTurnOnOrganization],
  render: (_props, context) => (
    <OrganizationsScreen viewport={context.viewport} state="after-team" />
  ),
});
