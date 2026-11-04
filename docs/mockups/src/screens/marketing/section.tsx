// The parts of a marketing section: the section between hairlines, its head
// with an eyebrow, a title, and a lead, the grid of cells divided by
// hairlines, and a row of actions.

import type { ReactNode } from "react";

export function Section({
  id,
  flush = false,
  className,
  children,
}: {
  id?: string;
  /** The section's grid runs to the rails; only the head keeps the gutter. */
  flush?: boolean;
  className?: string;
  children: ReactNode;
}) {
  const classes = ["mk-section", flush ? "is-flush" : "", className ?? ""];
  return (
    <section id={id} className={classes.filter(Boolean).join(" ")}>
      {children}
    </section>
  );
}

export function SectionHead({
  eyebrow,
  title,
  lead,
  page = false,
  center = false,
}: {
  eyebrow?: string;
  /** A title, or its sentences, which then start on a line each. */
  title: string | readonly string[];
  lead?: ReactNode;
  /** The head of the page, with the title as its h1. */
  page?: boolean;
  center?: boolean;
}) {
  const Title = page ? "h1" : "h2";
  return (
    <div className={`mk-head${center ? " is-center" : ""}`}>
      {eyebrow ? <p className="mk-eyebrow">{eyebrow}</p> : null}
      <Title className={`mk-title${page ? " is-page" : ""}`}>
        {typeof title === "string"
          ? title
          : title.map((line, index) => (
              <span key={line} className="mk-title-line">
                {index > 0 ? " " : null}
                {line}
              </span>
            ))}
      </Title>
      {lead ? <p className="mk-lead">{lead}</p> : null}
    </div>
  );
}

/** Cells divided by hairlines, from rail to rail. */
export function Cells({
  columns,
  className,
  children,
}: {
  columns: 2 | 3;
  className?: string;
  children: ReactNode;
}) {
  return (
    <div className={`mk-cells mk-cells-${columns} ${className ?? ""}`.trim()}>
      {children}
    </div>
  );
}

export function Actions({
  center = false,
  children,
}: {
  center?: boolean;
  children: ReactNode;
}) {
  return (
    <div className={`mk-actions${center ? " is-center" : ""}`}>{children}</div>
  );
}
