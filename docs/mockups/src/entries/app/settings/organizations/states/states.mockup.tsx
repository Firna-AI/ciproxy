import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../../components/paths.ts";
import { relatedMvp } from "../../../../../components/related-docs.ts";
import { orgAddress } from "../../../../../fixtures/org.ts";
import { OrganizationsScreen } from "../../../../../screens/settings/organizations.tsx";

const address = orgAddress("settings/organizations");

export const free = defineScreen({
  slug: "free",
  title: "One organization on Free",
  description:
    "The workspace mokly-ai on Free, in early September, with its one organization. Free includes one organization, so the notice offers Team. Add an organization stays, and a new organization joins inactive until the workspace moves to Team. The only organization has no Take out.",
  rationale:
    "The answer to item 48: Free includes one organization, and Team and Business several. Several GitHub organizations is a Team feature that ships at launch. The answer D to item 83: on Free, a new organization joins inactive, and the move to Team turns it on.",
  relatedDocs: relatedMvp("workspaces", "billing"),
  address,
  render: (_props, context) => (
    <OrganizationsScreen viewport={context.viewport} state="free" />
  ),
});

export const freeOver = defineScreen({
  slug: "free-over",
  title: "Two organizations on Free",
  description:
    "On 4 December, after Move to Free, mokly-ai is on Free and keeps mokly-ai and mokly-labs. Free includes one organization, so the banner of every page says that mokly-ai passed a limit of its plan and offers Team. Each of the two keeps its Take out. Add an organization stays, and a new organization joins inactive until Team.",
  rationale:
    "A workspace that holds more than one organization when it moves to Free keeps them all, and their jobs keep running. On Free, the number of organizations is a soft limit with the banner and the offer, as for the machine count.",
  relatedDocs: relatedMvp("workspaces", "metering", "billing"),
  address,
  useCasePaths: [
    paths.flowFreeKeepsOrganizations,
    paths.flowFreeAddsOrganization,
  ],
  render: (_props, context) => (
    <OrganizationsScreen viewport={context.viewport} state="free-over" />
  ),
});

export const oneOrganization = defineScreen({
  slug: "one-organization",
  title: "One organization on Team",
  description:
    "mokly-ai on Team in October, before mokly-labs joined. Team holds several organizations, so the page offers Add an organization. mokly-ai is the last organization of the workspace, so it has no Take out, as on Free.",
  rationale:
    "The answer D to item 51: the last organization of a workspace cannot be taken out, on every tier.",
  relatedDocs: relatedMvp("workspaces"),
  address,
  useCasePaths: [
    paths.flowAddOrganization,
    paths.flowChooseWorkspace,
    paths.flowApprovedLater,
  ],
  render: (_props, context) => (
    <OrganizationsScreen viewport={context.viewport} state="one-on-team" />
  ),
});

export const takeOut = defineScreen({
  slug: "take-out",
  title: "Take an organization out",
  description:
    "Cal Moore takes mokly-labs out of the workspace. No new job of mokly-labs starts on ciproxy, and its running jobs finish. ciproxy deletes its runner scale sets on GitHub and then removes the GitHub App from mokly-labs; the dialog names only the removal of the App. Its jobs stay in the job history, and mokly-labs can join any workspace again with a new install.",
  rationale:
    "The answer D to item 51: a take-out removes the App, so ciproxy keeps no install that no workspace serves, and the organization needs no new way to join. The rules of a removal of the App by an owner do not apply.",
  relatedDocs: relatedMvp("workspaces", "security"),
  address,
  render: (_props, context) => (
    <OrganizationsScreen viewport={context.viewport} state="take-out" />
  ),
});

export const removed = defineScreen({
  slug: "removed",
  title: "An organization whose App was removed",
  description:
    "A what-if of 20 November: an owner of mokly-labs removes the ciproxy GitHub App from mokly-labs on GitHub, without Take out. mokly-labs leaves the workspace and no new job of it starts on ciproxy. mokly-ai keeps running its jobs, and the workspace keeps its machines, its Boat account, and its subscription. For 30 days, the tab keeps mokly-labs as removed, with the day before which a new install keeps its job history, and offers Install again. ciproxy has no GitHub access left to mokly-labs, so its row has no repository access, no Repositories, and no Take out. mokly-ai is now the last linked organization, so it has no Take out either.",
  rationale:
    "app-removal.md: a removal takes only that organization out of its workspace, because the other organizations still use its machines, its Boat account, and its subscription. For 30 days, the Organizations tab shows the organization as removed, and a new install within that time keeps its job history. The rule of the last organization counts each linked organization, also an inactive or a suspended one, and not a removed one.",
  relatedDocs: relatedMvp("appRemoval", "workspaces"),
  address,
  useCasePaths: [paths.flowRemovedOrganization],
  render: (_props, context) => (
    <OrganizationsScreen viewport={context.viewport} state="removed" />
  ),
});
