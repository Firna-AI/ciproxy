import { defineScreen } from "@mokly/mokly";

import { relatedMvp } from "../../../../../components/related-docs.ts";
import { orgAddress } from "../../../../../fixtures/org.ts";
import { OrganizationsScreen } from "../../../../../screens/settings/organizations.tsx";

const address = orgAddress("settings/organizations");

export const free = defineScreen({
  slug: "free",
  title: "One organization on Free",
  description:
    "The workspace mokly-ai on Free, in early September, with its one organization. Free holds one organization, so the page offers Team in place of Add an organization, and the only organization has no Take out.",
  rationale:
    "The answer to item 48: Free allows one organization, and Team and Business allow several. Several GitHub organizations is a Team feature that ships at launch.",
  relatedDocs: relatedMvp("billing"),
  address,
  render: (_props, context) => (
    <OrganizationsScreen viewport={context.viewport} state="free" />
  ),
});

export const takeOut = defineScreen({
  slug: "take-out",
  title: "Take an organization out",
  description:
    "Cal Moore takes mokly-labs out of the workspace. No new job of mokly-labs starts on ciproxy, and its running jobs finish. Its jobs stay in the job history, and the GitHub App stays installed on mokly-labs until an owner removes it on GitHub.",
  rationale:
    "A design call of item 48: an Admin can take an organization out; its runners stop, and the App stays installed until an owner removes it on GitHub. Whether an organization that still has the App can join a workspace again is open in the docs.",
  relatedDocs: relatedMvp("security", "billing"),
  address,
  render: (_props, context) => (
    <OrganizationsScreen viewport={context.viewport} state="take-out" />
  ),
});
