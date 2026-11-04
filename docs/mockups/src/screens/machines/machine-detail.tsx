// One own machine: its slots and warm runners, health, cache, and the monthly
// cost that the savings report uses. A machine without KVM connects with no
// slots, so it cannot run jobs, and the page names the fix.

import { Pencil } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { Action, LinkAction, TextLink } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { Quantity, SlotStrip } from "../../components/figures.tsx";
import { NoKvmNotice } from "../../components/install-command.tsx";
import { PageHeader, Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { AppShell } from "../../components/shell/app-shell.tsx";
import { MachineStatusLabel } from "../../components/status.tsx";
import { dotJoin, monthly, unbroken } from "../../fixtures/format.ts";
import {
  ax42,
  ax42Size,
  machineScenes,
  type Machine,
  type MachineScene,
} from "../../fixtures/machines.ts";
import { ClearCacheDialog, MachineCache } from "./machine-cache.tsx";
import { MachineHealth } from "./machine-health.tsx";
import { MachineSlots } from "./machine-slots.tsx";
import "./machines.css";

export const machineStates = [
  "healthy",
  "draining",
  "offline",
  "no-slots",
  "clear-cache",
] as const;
export type MachineState = (typeof machineStates)[number];

/** The scene of each state, and the machine of that scene that it shows. */
const machineIn: Record<MachineState, [MachineScene, number]> = {
  healthy: ["steady", 0],
  draining: ["draining", 0],
  offline: ["offline", 2],
  "no-slots": ["noKvm", 2],
  "clear-cache": ["steady", 0],
};

function machineOf(state: MachineState): Machine {
  const [scene, index] = machineIn[state];
  const machine = machineScenes[scene][index];
  if (!machine) throw new Error(`No machine for ${state}`);
  return machine;
}

function HeaderActions({ state }: { state: MachineState }) {
  if (state === "offline") return <Action tone="danger">Remove machine</Action>;
  if (state === "draining") return <Action>Stop draining</Action>;
  if (state === "no-slots") return <Action tone="danger">Remove</Action>;
  return (
    <>
      <Action>Drain</Action>
      <Action tone="danger">Remove</Action>
    </>
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
  const noSlots = machine.slots.length === 0;
  return (
    <AppShell viewport={viewport} tier="team" active="machines">
      <PageHeader
        crumbs={[{ label: "Machines", to: paths.machines }]}
        title={machine.name}
        status={<MachineStatusLabel status={machine.status} />}
        subtitle={dotJoin(ax42.cpu, ax42Size, ax42.disk)}
        actions={<HeaderActions state={state} />}
      />
      {state === "draining" ? (
        <Notice tone="warning" title="Draining">
          3 jobs are finishing. The machine takes no new jobs, and it keeps no
          warm runners.
        </Notice>
      ) : null}
      {offline ? (
        <Notice tone="danger" title={`${machine.name} is offline`}>
          ciproxy lost contact with it {unbroken("14 minutes")} ago. Check that
          the machine is on and can reach ciproxy.dev.
        </Notice>
      ) : null}
      {state === "no-slots" ? (
        <NoKvmNotice
          name={machine.name}
          actions={
            <LinkAction to={paths.addMachine} size="sm">
              Show install command
            </LinkAction>
          }
        />
      ) : null}
      <div className="grid-2">
        <Panel
          title="Slots"
          action={<TextLink to={paths.jobsHistory}>Job history</TextLink>}
        >
          <div className="stack">
            <SlotsBody machine={machine} />
            {noSlots ? null : (
              <p className="caption">
                {ax42.slotsTotal} slots of {unbroken("2 vCPU")} and{" "}
                {unbroken("8 GB")}. A larger VM size takes several slots.
              </p>
            )}
            <Quantity
              label="Warm runners"
              value={machine.status === "healthy" ? machine.warmTarget : 0}
              caption="Ready for the next job when a slot is free"
            />
          </div>
        </Panel>
        <div className="stack">
          <MachineHealth machine={machine} />
          <MachineCache machine={machine} />
          <Panel title="Cost">
            <div className="spread">
              <div className="stack-tight">
                <p className="strong num">{monthly(ax42.monthlyCost)}</p>
                <p className="caption">The savings report uses this cost.</p>
              </div>
              <Action icon={Pencil} size="sm">
                Edit
              </Action>
            </div>
          </Panel>
        </div>
      </div>
      {state === "clear-cache" ? <ClearCacheDialog machine={machine} /> : null}
    </AppShell>
  );
}

/** The slot bar and what each slot runs, or why the machine shows none. */
function SlotsBody({ machine }: { machine: Machine }) {
  if (machine.status === "offline") {
    return (
      <p className="small muted">Slots show again when the machine is back.</p>
    );
  }
  if (machine.slots.length === 0) {
    return (
      <p className="small muted">
        No slots. Jobs run on your other machines and on Boat.
      </p>
    );
  }
  return (
    <>
      <SlotStrip slots={machine.slots} />
      <MachineSlots slots={machine.slots} />
    </>
  );
}
