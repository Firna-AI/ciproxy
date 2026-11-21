// The frame of a docs page, in the structure of Starlight: the site top bar
// with search, the sidebar of sections, the article, and "On this page" on the
// right. On mobile and in a narrow window, the sidebar moves behind the menu
// button and "On this page" becomes a bar above the article.

import type { ReactNode } from "react";
import { ChevronDown } from "lucide-react";
import { MockLink, type Viewport } from "@mokly/mokly";

import type { CataloguePath } from "../../components/paths.ts";
import { SiteHeader } from "../../components/site/site-header.tsx";
import { docsNav, docsNeighbours, type DocsNavItem } from "./docs-nav.ts";
import { Pagination, topId } from "./docs-parts.tsx";
import "./docs-shell.css";

/** A heading of the article in "On this page". */
export type TocItem = { id: string; label: string; nested?: boolean };

function SideLink({ item, active }: { item: DocsNavItem; active: boolean }) {
  const className = `docs-link${active ? " is-active" : ""}`;
  return item.to ? (
    <MockLink
      to={item.to}
      className={className}
      aria-current={active ? "page" : undefined}
    >
      {item.label}
    </MockLink>
  ) : (
    <span className={className}>{item.label}</span>
  );
}

function Sidebar({ page }: { page: CataloguePath }) {
  return (
    <aside className="docs-side">
      <nav className="docs-side-inner" aria-label="Docs">
        {docsNav.map((group) => (
          <div key={group.title} className="docs-group">
            <p className="docs-group-title">{group.title}</p>
            <ul className="docs-group-list">
              {group.items.map((item) => (
                <li key={item.label}>
                  <SideLink item={item} active={item.to === page} />
                </li>
              ))}
            </ul>
          </div>
        ))}
      </nav>
    </aside>
  );
}

function Toc({
  page,
  items,
}: {
  page: CataloguePath;
  items: readonly TocItem[];
}) {
  return (
    <aside className="docs-toc">
      <nav className="docs-toc-inner" aria-label="On this page">
        <p className="docs-toc-title">On this page</p>
        <ul className="docs-toc-list">
          <li>
            <MockLink
              to={page}
              fragment={topId}
              className="docs-toc-link is-current"
              aria-current="true"
            >
              Overview
            </MockLink>
          </li>
          {items.map((item) => (
            <li key={item.id}>
              <MockLink
                to={page}
                fragment={item.id}
                className={`docs-toc-link${item.nested ? " is-nested" : ""}`}
              >
                {item.label}
              </MockLink>
            </li>
          ))}
        </ul>
      </nav>
    </aside>
  );
}

/** "On this page" as a bar above the article, where the right column is gone. */
function TocBar() {
  return (
    <div className="docs-tocbar">
      <div className="docs-tocbar-inner">
        <span className="docs-tocbar-label">On this page</span>
        <span className="docs-tocbar-current">
          Overview
          <ChevronDown size={14} className="icon-muted" />
        </span>
      </div>
    </div>
  );
}

export function DocsShell({
  viewport,
  page,
  toc,
  overlay,
  children,
}: {
  viewport: Viewport;
  page: CataloguePath;
  toc: readonly TocItem[];
  /** A dialog over the page, such as search. */
  overlay?: ReactNode;
  children: ReactNode;
}) {
  const { prev, next } = docsNeighbours(page);
  const desktop = viewport === "desktop";
  return (
    <div className={`site-frame docs docs-${viewport}`}>
      <SiteHeader viewport={viewport} section="docs" />
      <div className="docs-body">
        {desktop ? <Sidebar page={page} /> : null}
        <main className="docs-main">
          <TocBar />
          <article className="docs-article">
            {children}
            <Pagination prev={prev} next={next} />
          </article>
        </main>
        {desktop ? <Toc page={page} items={toc} /> : null}
      </div>
      {overlay}
    </div>
  );
}
