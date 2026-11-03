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

/** A static dialog: a scrim over the page and a card, or a sheet on mobile. */
export function Dialog({
  title,
  children,
  footer,
}: {
  title: string;
  children: ReactNode;
  footer: ReactNode;
}) {
  return (
    <div className="dialog-scrim">
      <section className="dialog" role="dialog" aria-label={title}>
        <h2 className="dialog-title">{title}</h2>
        <div className="dialog-body">{children}</div>
        <div className="dialog-footer">{footer}</div>
      </section>
    </div>
  );
}
