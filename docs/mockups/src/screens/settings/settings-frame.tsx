// The frame of the settings pages: the app shell, the title, and the tabs.
// Only an Admin sees Settings. The Organizations tab holds the GitHub
// organizations of the workspace and the repository access of each. The
// Members tab counts the open requests for access, so the Admins learn of
// each one; ciproxy sends no email.

import type { ReactNode } from "react";
import type { Viewport } from "@mokly/mokly";

import { LinkTabs, PageHeader, type TabLink } from "../../components/page.tsx";
import { paths, type CataloguePath } from "../../components/paths.ts";
import { AppShell } from "../../components/shell/app-shell.tsx";
import type { TierKey } from "../../fixtures/org.ts";
import "./settings.css";

function tabs(openRequests: number): readonly TabLink[] {
  return [
    { label: "Labels", to: paths.settings },
    { label: "Overflow", to: paths.overflow },
    { label: "Organizations", to: paths.organizations },
    { label: "Members", to: paths.members, count: openRequests },
  ];
}

export function SettingsFrame({
  viewport,
  active,
  tier = "team",
  openRequests = 0,
  banner,
  children,
  overlay,
}: {
  viewport: Viewport;
  active: CataloguePath;
  /** The tier of the workspace, beside its name in the top bar. */
  tier?: TierKey;
  /** The requests for access that wait for an Admin. */
  openRequests?: number;
  /** The banner of every page, such as a passed soft limit. */
  banner?: ReactNode;
  children: ReactNode;
  /** A dialog over the page. */
  overlay?: ReactNode;
}) {
  return (
    <AppShell viewport={viewport} tier={tier} active="settings" banner={banner}>
      <PageHeader title="Settings" />
      <LinkTabs tabs={tabs(openRequests)} active={active} />
      {children}
      {overlay}
    </AppShell>
  );
}
