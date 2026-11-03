// One own machine: its slots and warm runners, health, cache, and the monthly
// cost that the savings report uses.

import { CircleCheck, CircleX, Pencil } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { Action, TextLink } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { Meter, Quantity, SlotStrip } from "../../components/figures.tsx";
import {
  DetailList,
  DetailRow,
  PageHeader,
  Panel,
} from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { AppShell } from "../../components/shell/app-shell.tsx";
import { MachineStatusBadge } from "../../components/status.tsx";
import { formatMoneyCents } from "../../fixtures/format.ts";
import { ax42, machineScenes, type Machine } from "../../fixtures/machines.ts";
import { MachineSlots } from "./machine-slots.tsx";
import "./machines.css";

export const machineStates = ["healthy", "draining", "offline"] as const;
export type MachineState = (typeof machineStates)[number];

function machineOf(state: MachineState): Machine {
  const scene = state === "healthy" ? "steady" : state;
  const index = state === "offline" ? 2 : 0;
  const machine = machineScenes[scene][index];
  if (!machine) throw new Error(`No machine for ${state}`);
  return machine;
}

function Check({ ok, children }: { ok: boolean; children: string }) {
  return (
    <span className="status-label">
      {ok ? (
        <CircleCheck size={15} className="state-passed" />
      ) : (
        <CircleX size={15} className="state-failed" />
      )}
      {children}
    </span>
  );
}

export function MachineDetailScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: MachineState;
}) {
  const machine = machineOf(state);
  const offline = state === "offline";
  return (
    <AppShell viewport={viewport} tier="team" active="machines">
      <PageHeader
        crumbs={[{ label: "Machines", to: paths.machines }]}
        title={machine.name}
        status={<MachineStatusBadge status={machine.status} />}
        subtitle={`${ax42.cpu} · ${ax42.threads} threads · ${ax42.memoryGb} GB · ${ax42.disk}`}
        actions={
          offline ? (
            <Action tone="danger">Remove machine</Action>
          ) : state === "draining" ? (
            <Action>Stop draining</Action>
          ) : (
            <>
              <Action>Drain</Action>
              <Action tone="danger">Remove</Action>
            </>
          )
        }
      />
      {state === "draining" ? (
        <Notice tone="warning" title="Draining">
          3 jobs are finishing. The machine takes no new jobs, and it keeps no
          warm runners.
        </Notice>
      ) : null}
      {offline ? (
        <Notice tone="danger" title={`${machine.name} is offline`}>
          The last heartbeat came 14 minutes ago. Check that the server is on
          and can reach ciproxy.dev.
        </Notice>
      ) : null}
      <div className="grid-2">
        <Panel
          title="Slots"
          action={<TextLink to={paths.jobsHistory}>Job history</TextLink>}
        >
          <div className="stack">
            {offline ? null : <SlotStrip slots={machine.slots} />}
            {offline ? (
              <p className="small muted">
                Slots show again when the machine is back.
              </p>
            ) : (
              <MachineSlots slots={machine.slots} />
            )}
            <p className="caption">
              6 slots of 2 vCPU and 8 GB. A larger shape takes several slots.
            </p>
            <Quantity
              label="Warm runners"
              value={state === "healthy" ? machine.warmTarget : 0}
              caption="Ready for the next job when a slot is free"
            />
          </div>
        </Panel>
        <div className="stack">
          <Panel title="Health">
            <DetailList>
              <DetailRow label="Heartbeat">
                <Check ok={!offline}>{machine.lastSeen}</Check>
              </DetailRow>
              <DetailRow label="KVM">
                <Check ok>Ready</Check>
              </DetailRow>
              <DetailRow label="Disk">
                <Check ok>312 GB free</Check>
              </DetailRow>
              <DetailRow label="Agent">
                <Check ok>{`${ax42.agent} · up to date`}</Check>
              </DetailRow>
            </DetailList>
          </Panel>
          <Panel title="Cache">
            <div className="stack">
              <Meter
                label="Cache disk"
                used={machine.cacheUsedGb}
                max={machine.cacheSizeGb}
                value={`${machine.cacheUsedGb} of ${machine.cacheSizeGb} GB`}
              />
              <DetailList>
                <DetailRow label="Hits, last 7 days">91%</DetailRow>
                <DetailRow label="Docker disks">
                  3 repositories · 96 GB
                </DetailRow>
              </DetailList>
            </div>
          </Panel>
          <Panel title="Cost">
            <div className="spread">
              <div className="stack-tight">
                <p className="strong num">
                  {formatMoneyCents(ax42.monthlyCost)} a month
                </p>
                <p className="caption">The savings report uses this cost.</p>
              </div>
              <Action icon={Pencil} size="sm">
                Edit
              </Action>
            </div>
          </Panel>
        </div>
      </div>
    </AppShell>
  );
}
