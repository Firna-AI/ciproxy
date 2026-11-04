// The own machines of the organization, their slots, warm runners, health,
// and cache, and the Boat account that takes overflow. Every role sees them.
// Only an Admin whom GitHub lets manage self-hosted runners adds a machine;
// another Admin sees who can. Only an Admin opens the overflow settings.

import { Plus, Server } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { LinkAction } from "../../components/actions.tsx";
import { Notice, ReadFailure, StateBlock } from "../../components/feedback.tsx";
import { NoRunnerRightNotice } from "../../components/install-command.tsx";
import { Kpi, KpiRow } from "../../components/kpis.tsx";
import { PageHeader, Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { canPay } from "../../components/role-actions.tsx";
import { AppShell } from "../../components/shell/app-shell.tsx";
import { usageBanner } from "../../components/usage-banner.tsx";
import {
  ax42,
  cacheHitsPercent,
  countSlots,
  machineScenes,
  offlineFor,
  type MachineScene,
} from "../../fixtures/machines.ts";
import { adminWithoutRight, type Role } from "../../fixtures/members.ts";
import { MachineTable, LoadingMachineTable } from "./machine-table.tsx";
import { OverflowPanel } from "./overflow-panel.tsx";
import "./machines.css";

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
  "over-limit": "fourMachines",
  loading: "steady",
  error: "steady",
};

export function MachinesScreen({
  viewport,
  state,
  role = "admin",
  runnerRight = true,
}: {
  viewport: Viewport;
  state: MachinesState;
  /** The role of the signed-in person. */
  role?: Role;
  /** GitHub lets the person manage self-hosted runners. */
  runnerRight?: boolean;
}) {
  const machines = machineScenes[sceneOf[state]];
  const admin = role === "admin";
  const canAdd = admin && runnerRight;
  const offline = machines.find((machine) => machine.status === "offline");
  return (
    <AppShell
      viewport={viewport}
      tier={state === "empty" ? "free" : "team"}
      active="machines"
      role={role}
      person={admin && !runnerRight ? adminWithoutRight : undefined}
      setup={
        state === "empty" ? { done: 1, total: 4, active: false } : undefined
      }
      banner={
        state === "over-limit"
          ? usageBanner("over-machines", "other", canPay(role))
          : undefined
      }
    >
      <PageHeader
        title="Machines"
        actions={
          state === "empty" || !canAdd ? undefined : (
            <LinkAction to={paths.addMachine} tone="primary" icon={Plus}>
              Add machine
            </LinkAction>
          )
        }
      />
      {admin && !runnerRight ? <NoRunnerRightNotice /> : null}
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
        canAdd={canAdd}
        admin={admin}
      />
    </AppShell>
  );
}

function MachinesBody({
  viewport,
  state,
  machines,
  canAdd,
  admin,
}: {
  viewport: Viewport;
  state: MachinesState;
  machines: (typeof machineScenes)[MachineScene];
  canAdd: boolean;
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
            canAdd ? (
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
  const online = machines.filter((machine) => machine.status !== "offline");
  const slots = online.length * ax42.slotsTotal;
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
        <Kpi
          label="Cache hits, 7 days"
          value={cacheHitsPercent}
          valueFormat={(value) => `${value}%`}
        />
      </KpiRow>
      <Panel title="Own machines" flush>
        <MachineTable
          viewport={viewport}
          machines={machines}
          readOnly={!admin}
        />
      </Panel>
      <OverflowPanel settings={admin} />
    </>
  );
}
