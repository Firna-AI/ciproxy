// The health of one machine. While it is offline, only the last contact is
// current: the other checks show their last values in grey, with a note. A
// machine without KVM shows that check as failed. The agent row says that
// the agent runs the newest release, or names the newest release beside an
// older one; the old value of an offline machine claims neither.

import type { ReactNode } from "react";

import { DetailList, DetailRow, Panel } from "../../components/page.tsx";
import { DotLabel } from "../../components/status.tsx";
import { dotJoin, unbroken } from "../../fixtures/format.ts";
import {
  latestAgentVersion,
  offlineFor,
  type Machine,
} from "../../fixtures/machines.ts";

function Check({ stale, children }: { stale: boolean; children: ReactNode }) {
  return stale ? (
    <DotLabel tone="idle">
      <span className="muted">{children}</span>
    </DotLabel>
  ) : (
    <DotLabel tone="ok">{children}</DotLabel>
  );
}

function AgentRelease({ machine }: { machine: Machine }) {
  const version = machine.agentVersion;
  if (machine.status === "offline") return <Check stale>{version}</Check>;
  if (version === latestAgentVersion) {
    return <Check stale={false}>{dotJoin(version, "up to date")}</Check>;
  }
  return (
    <DotLabel tone="warn">
      {dotJoin(version, unbroken(`${latestAgentVersion} available`))}
    </DotLabel>
  );
}

export function MachineHealth({ machine }: { machine: Machine }) {
  const offline = machine.status === "offline";
  return (
    <Panel title="Health">
      <DetailList>
        <DetailRow label="Last contact">
          <DotLabel tone={offline ? "err" : "ok"}>{machine.lastSeen}</DotLabel>
        </DetailRow>
        <DetailRow label="KVM">
          {machine.kvm ? (
            <Check stale={offline}>Ready</Check>
          ) : offline ? (
            <Check stale>Not available</Check>
          ) : (
            <DotLabel tone="err">Not available</DotLabel>
          )}
        </DetailRow>
        <DetailRow label="Disk">
          <Check stale={offline}>
            {unbroken(`${machine.freeDiskGb} GB`)} free
          </Check>
        </DetailRow>
        <DetailRow label="Agent">
          <AgentRelease machine={machine} />
        </DetailRow>
      </DetailList>
      {offline ? (
        <p className="caption health-note">
          KVM, disk, and agent show the values of the last contact, {offlineFor}{" "}
          ago.
        </p>
      ) : null}
    </Panel>
  );
}
