// The tabs of Settings and their addresses. Settings keeps one static page,
// so the query names the tab, and the repository access of one organization
// adds the organization ID. The Members tab counts the open requests for
// access, because ciproxy sends no email.

import { idParam, workspaceHref } from "../routing/pages.ts";

export const settingsTabs = [
  "labels",
  "overflow",
  "organizations",
  "members",
] as const;

export type SettingsTab = (typeof settingsTabs)[number];

const tabLabels: Readonly<Record<SettingsTab, string>> = {
  labels: "Labels",
  overflow: "Overflow",
  organizations: "Organizations",
  members: "Members",
};

/** The page of the query: a tab, or the repositories of one organization. */
export type SettingsRoute =
  | { kind: "tab"; tab: SettingsTab }
  | { kind: "repositories"; organizationId: string };

type QueryValue = string | readonly string[] | undefined;

function isTab(value: string): value is SettingsTab {
  return settingsTabs.some((tab) => tab === value);
}

/** The route of the query. A missing or unknown tab shows Labels. */
export function settingsRoute(query: {
  tab?: QueryValue;
  organization?: QueryValue;
}): SettingsRoute {
  const tab =
    typeof query.tab === "string" && isTab(query.tab) ? query.tab : "labels";
  const organizationId = idParam(query.organization);
  if (tab === "organizations" && organizationId !== null) {
    return { kind: "repositories", organizationId };
  }
  return { kind: "tab", tab };
}

export function settingsHref(workspaceId: string, tab: SettingsTab): string {
  return `${workspaceHref("settings", workspaceId)}&tab=${tab}`;
}

/** The repository access of one organization. */
export function repositoriesHref(
  workspaceId: string,
  organizationId: string,
): string {
  return `${settingsHref(workspaceId, "organizations")}&organization=${encodeURIComponent(organizationId)}`;
}

export type TabItem = {
  tab: SettingsTab;
  label: string;
  href: string;
  active: boolean;
  /** The open requests for access, on the Members tab. */
  count: number | null;
  /** The name that also says the count, such as "Members, 2 open requests". */
  accessibilityLabel: string;
};

export function tabItems(
  workspaceId: string,
  active: SettingsTab,
  openRequests: number | null,
): TabItem[] {
  return settingsTabs.map((tab) => {
    const label = tabLabels[tab];
    const count =
      tab === "members" && openRequests !== null && openRequests > 0
        ? openRequests
        : null;
    return {
      tab,
      label,
      href: settingsHref(workspaceId, tab),
      active: tab === active,
      count,
      accessibilityLabel:
        count === null
          ? label
          : `${label}, ${count} open ${count === 1 ? "request" : "requests"}`,
    };
  });
}
