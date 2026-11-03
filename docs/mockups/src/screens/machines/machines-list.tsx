// The own machines of the organization, their slots, warm runners, health,
// and cache, and the Boat account that takes overflow.

import type { ReactNode } from "react";
import { Plus, Server } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { LinkAction } from "../../components/actions.tsx";
import { Notice, ReadFailure, StateBlock } from "../../components/feedback.tsx";
import { Kpi, KpiRow } from "../../components/kpis.tsx";
import { PageHeader, Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { AppShell } from "../../components/shell/app-shell.tsx";
import {
  ax42,
  countSlots,
  machineScenes,
  type MachineScene,
} from "../../fixtures/machines.ts";
import { machineTrends } from "../../fixtures/trends.ts";
import { MachineTable, LoadingMachineTable } from "./machine-table.tsx";
import { OverflowPanel } from "./overflow-panel.tsx";
import "./machines.css";
import "./machines-dir.css";

export const machinesStates = [
  "steady",
  "empty",
  "offline",
  "over-limit",
  "loading",
  "error",
] as const;
export type MachinesState = (typeof machinesStates)[number];

const sceneOf: Record<MachinesState, MachineScene> = {
  steady: "steady",
  empty: "steady",
  offline: "offline",
  "over-limit": "fourNodes",
  loading: "steady",
  error: "steady",
};

function overLimitBanner(): ReactNode {
  return (
    <Notice
      tone="warning"
      title="4 own nodes on Team, which includes 3"
      actions={
        <LinkAction to={paths.usageOverNodes} tone="primary" size="sm">
          Add a node
        </LinkAction>
      }
    >
      Jobs keep running on all four. An extra node costs $20 a month.
    </Notice>
  );
}

export function MachinesScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: MachinesState;
}) {
  const machines = machineScenes[sceneOf[state]];
  return (
    <AppShell
      viewport={viewport}
      tier={state === "empty" ? "free" : "team"}
      active="machines"
      setup={
        state === "empty" ? { done: 1, total: 4, active: false } : undefined
      }
      banner={state === "over-limit" ? overLimitBanner() : undefined}
    >
      <PageHeader
        title="Machines"
        actions={
          state === "empty" ? undefined : (
            <LinkAction to={paths.addMachine} tone="primary" icon={Plus}>
              Add machine
            </LinkAction>
          )
        }
      />
      {state === "offline" ? (
        <Notice
          tone="danger"
          title="hetzner-ax42-3 is offline"
          actions={
            <LinkAction to={paths.machineOffline} size="sm">
              Open machine
            </LinkAction>
          }
        >
          It stopped sending heartbeats 14 minutes ago. New jobs run on your
          other machines and on Boat.
        </Notice>
      ) : null}
      <MachinesBody viewport={viewport} state={state} machines={machines} />
    </AppShell>
  );
}

function MachinesBody({
  viewport,
  state,
  machines,
}: {
  viewport: Viewport;
  state: MachinesState;
  machines: (typeof machineScenes)[MachineScene];
}) {
  if (state === "error") {
    return (
      <Panel>
        <ReadFailure retry={paths.machines} />
      </Panel>
    );
  }
  if (state === "empty") {
    return (
      <Panel>
        <StateBlock
          icon={Server}
          title="Add your first machine"
          action={
            <LinkAction to={paths.addMachine} tone="primary" icon={Plus}>
              Add machine
            </LinkAction>
          }
        >
          Run one command on a Linux server. Jobs start there first, and on Boat
          when it is busy.
        </StateBlock>
      </Panel>
    );
  }
  if (state === "loading") {
    return (
      <Panel title="Own machines" flush>
        <LoadingMachineTable />
      </Panel>
    );
  }
  const online = machines.filter((machine) => machine.status !== "offline");
  const slots = online.length * ax42.slotsTotal;
  const trends = state === "steady" ? machineTrends.steady : undefined;
  return (
    <>
      <KpiRow>
        <Kpi label="Machines" value={machines.length} />
        <Kpi
          label="Slots in use"
          value={countSlots(machines, "busy")}
          valueFormat={(value) => `${value} of ${slots}`}
          trend={trends?.busySlots}
        />
        <Kpi
          label="Warm runners"
          value={countSlots(machines, "warm")}
          trend={trends?.warmRunners}
        />
        <Kpi
          label="Cache hits, 7 days"
          value={91}
          valueFormat={(value) => `${value}%`}
          trend={trends?.cacheHits}
        />
      </KpiRow>
      <Panel title="Own machines" flush>
        <MachineTable viewport={viewport} machines={machines} />
      </Panel>
      <OverflowPanel />
    </>
  );
}
