// The parts of a docs article, after the Starlight components: the title and
// its lead, headings with ids for "On this page", steps on a rail, link cards,
// and the previous and next links. An aside is the shared Notice, which has a
// full hairline border instead of the left accent edge of a Starlight aside.

import type { ReactNode } from "react";
import { ArrowLeft, ArrowRight } from "lucide-react";
import { MockLink } from "@mokly/mokly";

import type { CataloguePath } from "../../components/paths.ts";
import type { DocsNavItem } from "./docs-nav.ts";
import "./docs-article.css";
import "./docs-blocks.css";

/** The id of the title, which "Overview" in "On this page" links to. */
export const topId = "_top";

export function DocsTitle({ title, lead }: { title: string; lead: ReactNode }) {
  return (
    <header className="docs-title-block">
      <h1 id={topId} className="docs-title">
        {title}
      </h1>
      <p className="docs-lead">{lead}</p>
    </header>
  );
}

export function DocsH2({ id, children }: { id: string; children: string }) {
  return (
    <h2 id={id} className="docs-h2">
      {children}
    </h2>
  );
}

export function Steps({ children }: { children: ReactNode }) {
  return <ol className="docs-steps">{children}</ol>;
}

export function Step({
  id,
  title,
  children,
}: {
  id: string;
  title: string;
  children: ReactNode;
}) {
  return (
    <li className="docs-step">
      <h3 id={id} className="docs-h3">
        {title}
      </h3>
      <div className="docs-flow">{children}</div>
    </li>
  );
}

export function CardGrid({ children }: { children: ReactNode }) {
  return <div className="docs-cards">{children}</div>;
}

export function LinkCard({
  title,
  text,
  to,
}: {
  title: string;
  text: string;
  to?: CataloguePath;
}) {
  const body = (
    <>
      <span className="docs-card-text">
        <span className="docs-card-title">{title}</span>
        <span className="docs-card-desc">{text}</span>
      </span>
      <ArrowRight size={16} className="docs-card-arrow" />
    </>
  );
  return to ? (
    <MockLink to={to} className="docs-card">
      {body}
    </MockLink>
  ) : (
    <span className="docs-card">{body}</span>
  );
}

function PagerLink({ item, dir }: { item: DocsNavItem; dir: "prev" | "next" }) {
  const body = (
    <>
      {dir === "prev" ? <ArrowLeft size={16} className="icon-muted" /> : null}
      <span className="docs-pager-text">
        <span className="caption">{dir === "prev" ? "Previous" : "Next"}</span>
        <span className="docs-pager-title">{item.label}</span>
      </span>
      {dir === "next" ? <ArrowRight size={16} className="icon-muted" /> : null}
    </>
  );
  const className = `docs-pager-link is-${dir}`;
  return item.to ? (
    <MockLink to={item.to} className={className}>
      {body}
    </MockLink>
  ) : (
    <span className={className}>{body}</span>
  );
}

export function Pagination({
  prev,
  next,
}: {
  prev?: DocsNavItem;
  next?: DocsNavItem;
}) {
  return (
    <nav className="docs-pager" aria-label="Previous and next page">
      {prev ? <PagerLink item={prev} dir="prev" /> : <span />}
      {next ? <PagerLink item={next} dir="next" /> : <span />}
    </nav>
  );
}
