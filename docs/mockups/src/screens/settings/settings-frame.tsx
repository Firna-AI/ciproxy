// The frame of the settings pages: the app shell, the title, and the tabs.

import type { ReactNode } from "react";
import type { Viewport } from "@mokly/mokly";

import { LinkTabs, PageHeader } from "../../components/page.tsx";
import { paths, type CataloguePath } from "../../components/paths.ts";
import { AppShell } from "../../components/shell/app-shell.tsx";
import "./settings.css";

const tabs = [
  { label: "Labels", to: paths.settings },
  { label: "Overflow", to: paths.overflow },
  { label: "Repositories", to: paths.repositories },
  { label: "Members", to: paths.members },
] as const;

export function SettingsFrame({
  viewport,
  active,
  children,
  overlay,
}: {
  viewport: Viewport;
  active: CataloguePath;
  children: ReactNode;
  /** A dialog over the page. */
  overlay?: ReactNode;
}) {
  return (
    <AppShell viewport={viewport} tier="team" active="settings">
      <PageHeader title="Settings" />
      <LinkTabs tabs={tabs} active={active} />
      {children}
      {overlay}
    </AppShell>
  );
}
