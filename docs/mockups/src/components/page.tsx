// Page building blocks: the page header with its breadcrumb, panels, the link
// tabs that switch between sibling pages, such as Live and History, and the
// details list. A tab with live data leads with a live dot, and a tab with
// open items, such as requests for access, ends with their count.

import type { ReactNode } from "react";
import { ChevronRight } from "lucide-react";
import { MockLink } from "@mokly/mokly";

import type { CataloguePath } from "./paths.ts";
import { Dot, Tag } from "./status.tsx";
import "./page.css";

export type Crumb = { label: string; to: CataloguePath };

export function PageHeader({
  title,
  subtitle,
  actions,
  crumbs,
  status,
}: {
  title: string;
  subtitle?: ReactNode;
  actions?: ReactNode;
  crumbs?: readonly Crumb[];
  status?: ReactNode;
}) {
  return (
    <header className="page-head">
      <div className="page-head-text">
        {crumbs ? (
          <nav className="crumbs" aria-label="Breadcrumb">
            {crumbs.map((crumb) => (
              <span key={crumb.to} className="row">
                <MockLink to={crumb.to} className="crumb">
                  {crumb.label}
                </MockLink>
                <ChevronRight size={14} className="icon-muted" />
              </span>
            ))}
          </nav>
        ) : null}
        <div className="page-title-row">
          <h1 className="page-title">{title}</h1>
          {status}
        </div>
        {subtitle ? <p className="page-subtitle">{subtitle}</p> : null}
      </div>
      {actions ? <div className="page-actions">{actions}</div> : null}
    </header>
  );
}

export function Panel({
  title,
  action,
  flush,
  children,
}: {
  title?: string;
  action?: ReactNode;
  /** Content runs to the panel edges, as a table does. */
  flush?: boolean;
  children: ReactNode;
}) {
  return (
    <section className={`panel${flush ? " panel-flush" : ""}`}>
      {title || action ? (
        <div className="panel-head">
          {title ? <h2 className="panel-title">{title}</h2> : <span />}
          {action}
        </div>
      ) : null}
      {children}
    </section>
  );
}

export type TabLink = {
  label: string;
  to: CataloguePath;
  /** The tab shows live data, so it leads with a live dot. */
  live?: boolean;
  /** Open items on the tab, such as requests for access. */
  count?: number;
};

export function LinkTabs({
  tabs,
  active,
}: {
  tabs: readonly TabLink[];
  active: CataloguePath;
}) {
  return (
    <nav className="link-tabs">
      {tabs.map((tab) => (
        <MockLink
          key={tab.to}
          to={tab.to}
          className={`link-tab${tab.to === active ? " is-active" : ""}`}
          aria-current={tab.to === active ? "page" : undefined}
        >
          {tab.live ? <Dot tone="run" /> : null}
          {tab.label}
          {tab.count ? <Tag tone="accent">{tab.count}</Tag> : null}
        </MockLink>
      ))}
    </nav>
  );
}

/** A label and its value, one row of a details list. */
export function DetailRow({
  label,
  children,
}: {
  label: string;
  children: ReactNode;
}) {
  return (
    <div className="detail-row">
      <dt>{label}</dt>
      <dd>{children}</dd>
    </div>
  );
}

export function DetailList({ children }: { children: ReactNode }) {
  return <dl className="detail-list">{children}</dl>;
}
