// Page building blocks: the page header, the breadcrumb, panels, and the link
// tabs that switch between sibling pages, such as Live and History. Inside a
// design direction, a live tab leads with a live dot.

import type { ReactNode } from "react";
import { ChevronRight } from "lucide-react";
import { MockLink } from "@mokly/mokly";

import { useDirection, useRoutes } from "../theme/direction.ts";
import type { CataloguePath } from "./paths.ts";
import { Dot } from "./status-dots.tsx";
import "./page.css";
import "./page-dir.css";

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
        <div className="row-wrap">
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
  count?: string;
  /** The tab shows live data; a direction marks it with a live dot. */
  live?: boolean;
};

export function LinkTabs({
  tabs,
  active,
}: {
  tabs: readonly TabLink[];
  active: CataloguePath;
}) {
  const direction = useDirection();
  const route = useRoutes();
  return (
    <nav className="link-tabs">
      {tabs.map((tab) => (
        <MockLink
          key={tab.to}
          to={route(tab.to)}
          className={`link-tab${tab.to === active ? " is-active" : ""}`}
          aria-current={tab.to === active ? "page" : undefined}
        >
          {direction && tab.live ? <Dot tone="run" live /> : null}
          {tab.label}
          {tab.count ? (
            <span className="link-tab-count">{tab.count}</span>
          ) : null}
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
