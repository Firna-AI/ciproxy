// The states of the Organizations tab: the tier of the workspace, its
// organizations, the banner of every page, and the open dialog. The December
// states add mokly-studio: it joins inactive on Free, it also waits for an
// Admin when the person who added it is no Admin now, and it runs once both
// holds clear. The other organizations keep running in every state.

import type { ReactNode } from "react";

import { inactiveBanner } from "../../components/inactive-banner.tsx";
import { paths, type CataloguePath } from "../../components/paths.ts";
import { usageBanner } from "../../components/usage-banner.tsx";
import {
  studioLinks,
  withStudio,
  type LinkedOrganization,
} from "../../fixtures/org-links.ts";
import type { TierKey } from "../../fixtures/org.ts";
import { organizationAccess } from "../../fixtures/settings.ts";
import { inactivePages } from "../billing/billing-scenes.tsx";
import {
  RenameWorkspaceDialog,
  TakeOutDialog,
  TurnOnDialog,
} from "./organization-dialogs.tsx";

export const organizationsStates = [
  "team",
  "free",
  "free-over",
  "one-on-team",
  "take-out",
  "rename",
  "inactive-plan",
  "both-holds",
  "inactive-admin",
  "turn-on",
  "after-team",
] as const;
export type OrganizationsState = (typeof organizationsStates)[number];

export type OrganizationsScene = {
  tier: TierKey;
  list: readonly LinkedOrganization[];
  /** The banner of every page, such as a passed soft limit. */
  banner?: ReactNode;
  /** A dialog over the page. */
  overlay?: ReactNode;
  /** The Turn on dialog of an inactive organization, when it exists. */
  turnOn?: CataloguePath;
};

/** The first organization alone, before mokly-labs joins. */
const firstOnly = organizationAccess.slice(0, 1);

/** mokly-studio with its holds, and the banner that says why. */
function held(
  tier: TierKey,
  link: LinkedOrganization,
  turnOn?: CataloguePath,
): OrganizationsScene {
  return {
    tier,
    list: withStudio(link),
    banner: inactiveBanner(link, inactivePages, turnOn),
    turnOn,
  };
}

export function organizationsScene(
  state: OrganizationsState,
): OrganizationsScene {
  switch (state) {
    case "team":
      return { tier: "team", list: organizationAccess };
    case "free":
      return { tier: "free", list: firstOnly };
    case "free-over":
      return {
        tier: "free",
        list: organizationAccess,
        banner: usageBanner("free-organizations", "other"),
      };
    case "one-on-team":
      return { tier: "team", list: firstOnly };
    case "take-out":
      return {
        tier: "team",
        list: organizationAccess,
        overlay: <TakeOutDialog />,
      };
    case "rename":
      return {
        tier: "team",
        list: organizationAccess,
        overlay: <RenameWorkspaceDialog />,
      };
    case "inactive-plan":
      return held("free", studioLinks.plan);
    case "both-holds":
      return held("free", studioLinks.both);
    case "inactive-admin":
      return held("team", studioLinks.admin, paths.inactiveTurnOn);
    case "turn-on":
      return { ...held("team", studioLinks.admin), overlay: <TurnOnDialog /> };
    case "after-team":
      return { tier: "team", list: withStudio(studioLinks.active) };
  }
}
