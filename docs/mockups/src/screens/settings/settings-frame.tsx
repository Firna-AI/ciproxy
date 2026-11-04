// The frame of the settings pages: the app shell, the title, and the tabs.
// Only an Admin sees Settings. The Members tab counts the open requests for
// access, so the Admins learn of each one; ciproxy sends no email.

import type { ReactNode } from "react";
import type { Viewport } from "@mokly/mokly";

import { LinkTabs, PageHeader, type TabLink } from "../../components/page.tsx";
import { paths, type CataloguePath } from "../../components/paths.ts";
import { AppShell } from "../../components/shell/app-shell.tsx";
import "./settings.css";

function tabs(openRequests: number): readonly TabLink[] {
  return [
    { label: "Labels", to: paths.settings },
    { label: "Overflow", to: paths.overflow },
    { label: "Repositories", to: paths.repositories },
    { label: "Members", to: paths.members, count: openRequests },
  ];
}

export function SettingsFrame({
  viewport,
  active,
  openRequests = 0,
  children,
  overlay,
}: {
  viewport: Viewport;
  active: CataloguePath;
  /** The requests for access that wait for an Admin. */
  openRequests?: number;
  children: ReactNode;
  /** A dialog over the page. */
  overlay?: ReactNode;
}) {
  return (
    <AppShell viewport={viewport} tier="team" active="settings">
      <PageHeader title="Settings" />
      <LinkTabs tabs={tabs(openRequests)} active={active} />
      {children}
      {overlay}
    </AppShell>
  );
}
