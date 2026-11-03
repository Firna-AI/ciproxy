// Notices, state blocks, and the failed-read state. A notice is a tinted box
// with a full border and a leading icon; it never carries an accent edge.

import type { ReactNode } from "react";
import type { LucideIcon } from "lucide-react";
import { CircleAlert, CircleCheck, Info, TriangleAlert } from "lucide-react";

import { LinkAction } from "./actions.tsx";
import type { CataloguePath } from "./paths.ts";
import "./feedback.css";

export type NoticeTone = "info" | "success" | "warning" | "danger";

const noticeIcons: Record<NoticeTone, LucideIcon> = {
  info: Info,
  success: CircleCheck,
  warning: TriangleAlert,
  danger: CircleAlert,
};

export function Notice({
  tone,
  title,
  children,
  actions,
}: {
  tone: NoticeTone;
  title: string;
  children?: ReactNode;
  actions?: ReactNode;
}) {
  const Icon = noticeIcons[tone];
  return (
    <div
      className={`notice notice-${tone}`}
      role={tone === "danger" ? "alert" : "status"}
    >
      <Icon size={18} className="notice-icon" />
      <div className="notice-text">
        <p className="notice-title">{title}</p>
        {children ? <div className="notice-body">{children}</div> : null}
      </div>
      {actions ? <div className="notice-actions">{actions}</div> : null}
    </div>
  );
}

export type StateTone = "neutral" | "primary" | "warning" | "danger";

/** The centred block of an empty, first-run, or failed state. */
export function StateBlock({
  icon: Icon,
  tone = "primary",
  title,
  children,
  action,
}: {
  icon: LucideIcon;
  tone?: StateTone;
  title: string;
  children?: ReactNode;
  action?: ReactNode;
}) {
  return (
    <div className="state-block">
      <span className={`state-icon state-icon-${tone}`} aria-hidden="true">
        <Icon size={22} />
      </span>
      <h2 className="state-title">{title}</h2>
      {children ? <div className="state-body">{children}</div> : null}
      {action ? <div className="row-wrap state-action">{action}</div> : null}
    </div>
  );
}

/** The one state a failed page read shows, with Try again. */
export function ReadFailure({ retry }: { retry: CataloguePath }) {
  return (
    <StateBlock
      icon={CircleAlert}
      tone="danger"
      title="Unable to load this page"
      action={
        <LinkAction to={retry} tone="primary">
          Try again
        </LinkAction>
      }
    />
  );
}
