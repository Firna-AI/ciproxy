// Status: a small dot beside its word, never colour alone and never a pill.
// Machine names use the monospace face. A tag is a small bordered label, such
// as the tier beside the organization or the current plan.

import type { ReactNode } from "react";
import { Cloud, Server } from "lucide-react";
import { StatusDot } from "@firna/ui/status-dot";

import type { JobState, Where } from "../fixtures/jobs.ts";
import type { MachineStatus } from "../fixtures/machines.ts";
import { statusColors, type StatusTone } from "../theme/graphite.ts";
import "./status.css";

type StateLook = { label: string; tone: StatusTone };

const jobStates: Readonly<Record<JobState, StateLook>> = {
  running: { label: "Running", tone: "run" },
  waiting: { label: "Waiting", tone: "warn" },
  passed: { label: "Passed", tone: "ok" },
  failed: { label: "Failed", tone: "err" },
  cancelled: { label: "Cancelled", tone: "idle" },
};

const machineStates: Readonly<Record<MachineStatus, StateLook>> = {
  healthy: { label: "Healthy", tone: "ok" },
  draining: { label: "Draining", tone: "warn" },
  offline: { label: "Offline", tone: "err" },
};

/** The dot alone. Pass a label when no word beside it names the state. */
export function Dot({ tone, label }: { tone: StatusTone; label?: string }) {
  return (
    <span className="dot">
      <StatusDot color={statusColors[tone]} size="sm" label={label} />
    </span>
  );
}

export function DotLabel({
  tone,
  children,
}: {
  tone: StatusTone;
  children: ReactNode;
}) {
  return (
    <span className="dot-label">
      <Dot tone={tone} />
      <span>{children}</span>
    </span>
  );
}

/** The state of a job as a dot that names itself, for a row that leads with it. */
export function JobStateDot({ state }: { state: JobState }) {
  const { label, tone } = jobStates[state];
  return <Dot tone={tone} label={label} />;
}

export function JobStateLabel({ state }: { state: JobState }) {
  const { label, tone } = jobStates[state];
  return <DotLabel tone={tone}>{label}</DotLabel>;
}

export function MachineStatusLabel({ status }: { status: MachineStatus }) {
  const { label, tone } = machineStates[status];
  return <DotLabel tone={tone}>{label}</DotLabel>;
}

/** Where a job runs or ran: an own machine and, while it runs, its slot. */
export function WhereLabel({
  where,
  slot = false,
}: {
  where: Where;
  slot?: boolean;
}) {
  if (where.kind === "none") {
    return <span className="muted">Waiting for a runner</span>;
  }
  if (where.kind === "boat") {
    return (
      <span className="where">
        <Cloud size={14} className="icon-muted" />
        <span>Boat</span>
      </span>
    );
  }
  return (
    <span className="where">
      <Server size={14} className="icon-muted" />
      <span className="mono where-machine">{where.machine}</span>
      {slot ? <span className="muted">slot {where.slot}</span> : null}
    </span>
  );
}

/** A small bordered label. The accent tone fills it with the ink. */
export function Tag({
  tone = "neutral",
  children,
}: {
  tone?: "neutral" | "accent";
  children: ReactNode;
}) {
  return <span className={`tag tag-${tone}`}>{children}</span>;
}
