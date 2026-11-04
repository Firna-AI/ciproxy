// Notices, state blocks, the failed-read state, and the busy spinner. A notice
// is a box with a full hairline border and a leading icon; it never carries an
// accent edge.

import type { ReactNode } from "react";
import type { LucideIcon } from "lucide-react";
import { CircleAlert, Info, TriangleAlert } from "lucide-react";
import { Spinner } from "@firna/ui/spinner";

import { theme } from "../theme/graphite.ts";
import { LinkAction } from "./actions.tsx";
import type { CataloguePath } from "./paths.ts";
import "./feedback.css";

export type NoticeTone = "info" | "warning" | "danger";

const noticeIcons: Record<NoticeTone, LucideIcon> = {
  info: Info,
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
      <Icon size={16} className="notice-icon" />
      <div className="notice-text">
        <p className="notice-title">{title}</p>
        {children ? <div className="notice-body">{children}</div> : null}
      </div>
      {actions ? <div className="notice-actions">{actions}</div> : null}
    </div>
  );
}

export type StateTone = "neutral" | "success" | "warning" | "danger";

/** The centred block of an empty, first-run, finished, or failed state. */
export function StateBlock({
  icon: Icon,
  tone = "neutral",
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
        <Icon size={18} />
      </span>
      <h2 className="state-title">{title}</h2>
      {children ? <div className="state-body">{children}</div> : null}
      {action ? <div className="row-wrap state-action">{action}</div> : null}
    </div>
  );
}

/** The state of a failed page read: what failed, and Try again. */
export function ReadFailure({
  subject,
  retry,
}: {
  /** What did not load, such as "jobs" or "plan and usage". */
  subject: string;
  retry: CataloguePath;
}) {
  return (
    <StateBlock
      icon={CircleAlert}
      tone="danger"
      title={`Unable to load ${subject}`}
      action={
        <LinkAction to={retry} tone="primary">
          Try again
        </LinkAction>
      }
    >
      Jobs keep running.
    </StateBlock>
  );
}

/** A spinner in the muted ink, for a step that is still working. */
export function Busy({ size = 16 }: { size?: number }) {
  return <Spinner size={size} color={theme.colors.muted} />;
}
