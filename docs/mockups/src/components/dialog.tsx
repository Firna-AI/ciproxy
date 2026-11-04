// A static dialog over a page: a scrim and a card on desktop, a sheet on
// mobile. The page renders it after its content, as the open state of a
// screen, such as Edit label, Clear cache, or Move to Free.

import type { ReactNode } from "react";

import "./dialog.css";

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
