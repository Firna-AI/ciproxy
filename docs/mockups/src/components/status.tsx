// Status labels. Each state carries an icon and a word, never colour alone.
// Inside a design direction, the state is a dot beside its word.

import {
  CircleCheck,
  CircleSlash,
  CircleX,
  Cloud,
  Hourglass,
  Server,
} from "lucide-react";
import { Badge } from "@firna/ui/badge";
import { Spinner } from "@firna/ui/spinner";

import type { JobState, Where } from "../fixtures/jobs.ts";
import type { MachineStatus } from "../fixtures/machines.ts";
import { useDirection } from "../theme/direction.ts";
import { theme } from "../theme/firna.tsx";
import {
  Dot,
  DotLabel,
  jobTones,
  machineTones,
  WhereText,
} from "./status-dots.tsx";
import "./status.css";

const jobStateLabels: Record<JobState, string> = {
  running: "Running",
  waiting: "Waiting",
  passed: "Passed",
  failed: "Failed",
  cancelled: "Cancelled",
};

export function JobStateIcon({ state }: { state: JobState }) {
  const direction = useDirection();
  if (direction) {
    return (
      <Dot
        tone={jobTones[state]}
        live={state === "running"}
        label={jobStateLabels[state]}
      />
    );
  }
  switch (state) {
    case "running":
      return (
        <Spinner
          size={14}
          color={theme.colors.primary}
          accessibilityLabel="Running"
        />
      );
    case "waiting":
      return (
        <Hourglass size={15} className="state-waiting" aria-label="Waiting" />
      );
    case "passed":
      return (
        <CircleCheck size={16} className="state-passed" aria-label="Passed" />
      );
    case "failed":
      return <CircleX size={16} className="state-failed" aria-label="Failed" />;
    case "cancelled":
      return (
        <CircleSlash
          size={16}
          className="state-cancelled"
          aria-label="Cancelled"
        />
      );
  }
}

export function JobStateLabel({ state }: { state: JobState }) {
  const direction = useDirection();
  if (direction) {
    return (
      <DotLabel tone={jobTones[state]} live={state === "running"}>
        {jobStateLabels[state]}
      </DotLabel>
    );
  }
  return (
    <span className="status-label">
      <JobStateIcon state={state} />
      <span>{jobStateLabels[state]}</span>
    </span>
  );
}

/** Where a job ran or runs: an own machine and its slot, or Boat. */
export function WhereLabel({
  where,
  slot = false,
}: {
  where: Where;
  slot?: boolean;
}) {
  const direction = useDirection();
  if (direction) return <WhereText where={where} slot={slot} />;
  if (where.kind === "none") {
    return <span className="muted">Waiting for a runner</span>;
  }
  if (where.kind === "boat") {
    return (
      <span className="status-label">
        <Cloud size={15} className="icon-muted" />
        <span>Boat</span>
      </span>
    );
  }
  return (
    <span className="status-label">
      <Server size={15} className="icon-muted" />
      <span>
        {where.machine}
        {slot ? <span className="muted"> · slot {where.slot}</span> : null}
      </span>
    </span>
  );
}

const machineBadges: Record<
  MachineStatus,
  { label: string; tone: "primary" | "warning" | "danger" }
> = {
  healthy: { label: "Healthy", tone: "primary" },
  draining: { label: "Draining", tone: "warning" },
  offline: { label: "Offline", tone: "danger" },
};

export function MachineStatusBadge({ status }: { status: MachineStatus }) {
  const direction = useDirection();
  const badge = machineBadges[status];
  if (direction) {
    return <DotLabel tone={machineTones[status]}>{badge.label}</DotLabel>;
  }
  return (
    <span className="badge-wrap">
      <Badge tone={badge.tone} dot size="sm">
        {badge.label}
      </Badge>
    </span>
  );
}
