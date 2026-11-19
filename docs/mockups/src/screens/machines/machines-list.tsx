// The own machines of the workspace, their slots, warm runners, health, and
// cache, and the Boat account that takes overflow. Every role sees them. Only
// an Admin adds a machine and opens the overflow settings. A failed read of
// the Overflow panel shows in the panel, and the machines stay on the page.
// The slot figures count only the machines that run jobs or can take them,
// never an offline machine or one that gets its first job image.

import { Plus, Server } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { LinkAction } from "../../components/actions.tsx";
import { Notice, ReadFailure, StateBlock } from "../../components/feedback.tsx";
import { Kpi, KpiRow } from "../../components/kpis.tsx";
import { PageHeader, Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { canPay } from "../../components/role-actions.tsx";
import { AppShell } from "../../components/shell/app-shell.tsx";
import { usageBanner } from "../../components/usage-banner.tsx";
import {
  countSlots,
  machineScenes,
  offlineFor,
  type Machine,
} from "../../fixtures/machines.ts";
import type { Role } from "../../fixtures/members.ts";
import { newMachineScene } from "../../fixtures/new-machine.ts";
import { CacheHitsKpi } from "./cache-hits.tsx";
import {
  LoadingMachineTable,
  MachineTable,
  slotsLive,
} from "./machine-table.tsx";
import { OverflowPanel } from "./overflow-panel.tsx";
import "./machines.css";

export const machinesStates = [
  "steady",
  "empty",
  "offline",
  "over-limit",
  "cache-unknown",
  "no-cache-measured",
  "new-machine",
  "loading",
  "error",
  "overflow-refresh",
] as const;
export type MachinesState = (typeof machinesStates)[number];

const machinesOf: Record<MachinesState, readonly Machine[]> = {
  steady: machineScenes.steady,
  empty: machineScenes.steady,
  offline: machineScenes.offline,
  "over-limit": machineScenes.fourMachines,
  "cache-unknown": machineScenes.cacheUnknown,
  "no-cache-measured": machineScenes.noCacheMeasured,
  "new-machine": newMachineScene,
  loading: machineScenes.steady,
  error: machineScenes.steady,
  "overflow-refresh": machineScenes.steady,
};

/** A fourth machine passes the three machines of Team. */
function overMachines(state: MachinesState): boolean {
  return state === "over-limit" || state === "new-machine";
}

export function MachinesScreen({
  viewport,
  state,
  role = "admin",
}: {
  viewport: Viewport;
  state: MachinesState;
  /** The role of the signed-in person. */
  role?: Role;
}) {
  const machines = machinesOf[state];
  const admin = role === "admin";
  const offline = machines.find((machine) => machine.status === "offline");
  return (
    <AppShell
      viewport={viewport}
      tier={state === "empty" ? "free" : "team"}
      active="machines"
      role={role}
      setup={
        state === "empty" ? { done: 1, total: 4, active: false } : undefined
      }
      banner={
        overMachines(state)
          ? usageBanner("over-machines", "other", canPay(role))
          : undefined
      }
    >
      <PageHeader
        title="Machines"
        actions={
          state === "empty" || !admin ? undefined : (
            <LinkAction to={paths.addMachine} tone="primary" icon={Plus}>
              Add machine
            </LinkAction>
          )
        }
      />
      {offline ? (
        <Notice
          tone="danger"
          title={`${offline.name} is offline`}
          actions={
            <LinkAction to={paths.machineOffline} size="sm">
              Open machine
            </LinkAction>
          }
        >
          ciproxy lost contact with it {offlineFor} ago. New jobs run on your
          other machines and on Boat.
        </Notice>
      ) : null}
      <MachinesBody
        viewport={viewport}
        state={state}
        machines={machines}
        admin={admin}
      />
    </AppShell>
  );
}

function MachinesBody({
  viewport,
  state,
  machines,
  admin,
}: {
  viewport: Viewport;
  state: MachinesState;
  machines: readonly Machine[];
  admin: boolean;
}) {
  if (state === "error") {
    return (
      <Panel>
        <ReadFailure subject="machines" retry={paths.machines} />
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
            admin ? (
              <LinkAction to={paths.addMachine} tone="primary" icon={Plus}>
                Add machine
              </LinkAction>
            ) : undefined
          }
        >
          Run one command on a Linux machine. Jobs start there first, and on
          Boat when it is busy.
        </StateBlock>
      </Panel>
    );
  }
  if (state === "loading") {
    return (
      <Panel title="Own machines" flush>
        <LoadingMachineTable viewport={viewport} />
      </Panel>
    );
  }
  const slots = machines
    .filter(slotsLive)
    .reduce((sum, machine) => sum + machine.slots.length, 0);
  return (
    <>
      <KpiRow>
        <Kpi label="Machines" value={machines.length} />
        <Kpi
          label="Slots in use"
          value={countSlots(machines, "busy")}
          valueFormat={(value) => `${value} of ${slots}`}
        />
        <Kpi label="Warm runners" value={countSlots(machines, "warm")} />
        <CacheHitsKpi machines={machines} />
      </KpiRow>
      <Panel title="Own machines" flush>
        <MachineTable
          viewport={viewport}
          machines={machines}
          readOnly={!admin}
        />
      </Panel>
      <OverflowPanel
        settings={admin}
        refreshFailed={state === "overflow-refresh"}
      />
    </>
  );
}
