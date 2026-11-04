// The health of one machine. While it is offline, only the last contact is
// current: the other checks show their last values in grey, with a note. A
// machine without KVM shows that check as failed.

import type { ReactNode } from "react";

import { DetailList, DetailRow, Panel } from "../../components/page.tsx";
import { DotLabel } from "../../components/status.tsx";
import { dotJoin, unbroken } from "../../fixtures/format.ts";
import { ax42, type Machine } from "../../fixtures/machines.ts";

function Check({ stale, children }: { stale: boolean; children: ReactNode }) {
  return stale ? (
    <DotLabel tone="idle">
      <span className="muted">{children}</span>
    </DotLabel>
  ) : (
    <DotLabel tone="ok">{children}</DotLabel>
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
          ) : (
            <DotLabel tone="err">Not available</DotLabel>
          )}
        </DetailRow>
        <DetailRow label="Disk">
          <Check stale={offline}>{unbroken("312 GB")} free</Check>
        </DetailRow>
        <DetailRow label="Agent">
          <Check stale={offline}>
            {offline ? ax42.agent : dotJoin(ax42.agent, "up to date")}
          </Check>
        </DetailRow>
      </DetailList>
      {offline ? (
        <p className="caption health-note">
          KVM, disk, and agent show the values of the last contact,{" "}
          {unbroken("14 minutes")} ago.
        </p>
      ) : null}
    </Panel>
  );
}
