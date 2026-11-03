// Status in a design direction: a small dot beside its word, never a pill.
// A live state glows where the direction sets a glow. Machine names use the
// monospace face.

import type { ReactNode } from "react";
import { Cloud, Server } from "lucide-react";
import { StatusDot } from "@firna/ui/status-dot";

import type { JobState, Where } from "../fixtures/jobs.ts";
import type { MachineStatus } from "../fixtures/machines.ts";
import { useDirection, type StatusTone } from "../theme/direction.ts";

export const jobTones: Readonly<Record<JobState, StatusTone>> = {
  running: "run",
  waiting: "warn",
  passed: "ok",
  failed: "err",
  cancelled: "idle",
};

export const machineTones: Readonly<Record<MachineStatus, StatusTone>> = {
  healthy: "ok",
  draining: "warn",
  offline: "err",
};

/** The dot alone. Pass a label when no word beside it names the state. */
export function Dot({
  tone,
  live = false,
  label,
}: {
  tone: StatusTone;
  live?: boolean;
  label?: string;
}) {
  const direction = useDirection();
  return (
    <span className={`dot-wrap dot-${tone}${live ? " is-live" : ""}`}>
      <StatusDot color={direction?.status[tone]} size="sm" label={label} />
    </span>
  );
}

export function DotLabel({
  tone,
  live = false,
  children,
}: {
  tone: StatusTone;
  live?: boolean;
  children: ReactNode;
}) {
  return (
    <span className="dot-label">
      <Dot tone={tone} live={live} />
      <span>{children}</span>
    </span>
  );
}

/** Where a job runs: an own machine and its slot, or Boat. */
export function WhereText({ where, slot }: { where: Where; slot: boolean }) {
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
