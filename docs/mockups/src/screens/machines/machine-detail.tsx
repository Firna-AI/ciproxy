// One own machine: its slots and warm runners, health, cache, and the monthly
// cost that the savings report uses. A machine without KVM connects with no
// slots, so it cannot run jobs, and the page names the fix. A machine without
// room for a slot says why too. A machine whose disk holds fewer jobs than
// its slots says so, as design call 34 says. After Clear cache, or a drain
// that waits for the machine to connect again, a notice says so. Every role
// sees the machine; only an Admin drains, removes, or changes it.
// The warm runner count is the saved count in every state, also while the
// machine drains, is offline, or has no slots; the slots show the warm
// runners that are ready.

import { Pencil } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { Action, TextLink } from "../../components/actions.tsx";
import { Quantity, SlotStrip } from "../../components/figures.tsx";
import { PageHeader, Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { canPay } from "../../components/role-actions.tsx";
import { AppShell } from "../../components/shell/app-shell.tsx";
import { MachineStatusLabel } from "../../components/status.tsx";
import { usageBanner } from "../../components/usage-banner.tsx";
import { dotJoin, monthly } from "../../fixtures/format.ts";
import { hardwareSize, slotSize } from "../../fixtures/hardware.ts";
import {
  machineScenes,
  type Machine,
  type MachineScene,
} from "../../fixtures/machines.ts";
import type { Role } from "../../fixtures/members.ts";
import { defaultLabelJobs } from "../../fixtures/settings.ts";
import { sizePhrase } from "../../fixtures/sizes.ts";
import { ClearCacheDialog, MachineCache } from "./machine-cache.tsx";
import { MachineHealth } from "./machine-health.tsx";
import { MachineNotice } from "./machine-notices.tsx";
import { MachineSlots } from "./machine-slots.tsx";
import "./machines.css";

export const machineStates = [
  "healthy",
  "draining",
  "offline",
  "no-slots",
  "disk-limit",
  "clear-cache",
  "clear-requested",
  "saved-offline",
  "no-room",
  "agent-behind",
] as const;
export type MachineState = (typeof machineStates)[number];

/** The scene of each state, and the machine of that scene that it shows. */
const machineIn: Record<MachineState, [MachineScene, number]> = {
  healthy: ["steady", 0],
  draining: ["draining", 0],
  offline: ["offline", 2],
  "no-slots": ["noKvm", 2],
  "disk-limit": ["fourMachines", 3],
  "clear-cache": ["steady", 0],
  "clear-requested": ["steady", 0],
  "saved-offline": ["drainSaved", 0],
  "no-room": ["tooSmall", 3],
  "agent-behind": ["agentBehind", 0],
};

function machineOf(state: MachineState): Machine {
  const [scene, index] = machineIn[state];
  const machine = machineScenes[scene][index];
  if (!machine) throw new Error(`No machine for ${state}`);
  return machine;
}

function HeaderActions({ state }: { state: MachineState }) {
  if (state === "offline") return <Action tone="danger">Remove machine</Action>;
  if (state === "draining" || state === "saved-offline") {
    return <Action>Stop draining</Action>;
  }
  if (state === "no-slots" || state === "no-room") {
    return <Action tone="danger">Remove</Action>;
  }
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
  role = "admin",
}: {
  viewport: Viewport;
  state: MachineState;
  /** The role of the signed-in person: only an Admin changes the machine. */
  role?: Role;
}) {
  const machine = machineOf(state);
  const { hardware } = machine;
  const noSlots = machine.slots.length === 0;
  const admin = role === "admin";
  return (
    <AppShell
      viewport={viewport}
      tier="team"
      active="machines"
      role={role}
      banner={
        state === "disk-limit" || state === "no-room"
          ? usageBanner("over-machines", "other", canPay(role))
          : undefined
      }
    >
      <PageHeader
        crumbs={[{ label: "Machines", to: paths.machines }]}
        title={machine.name}
        status={<MachineStatusLabel status={machine.status} />}
        subtitle={dotJoin(hardware.cpu, hardwareSize(hardware), hardware.disk)}
        actions={admin ? <HeaderActions state={state} /> : undefined}
      />
      {state === "healthy" ||
      state === "clear-cache" ||
      state === "agent-behind" ? null : (
        <MachineNotice state={state} machine={machine} admin={admin} />
      )}
      <div className="grid-2">
        <Panel
          title="Slots"
          action={<TextLink to={paths.jobsHistory}>Job history</TextLink>}
        >
          <div className="stack">
            <SlotsBody machine={machine} />
            {noSlots ? null : (
              <p className="caption">
                {machine.slots.length} slots of {sizePhrase(slotSize)}. A larger
                VM size takes several slots.
              </p>
            )}
            <Quantity
              label="Warm runners"
              value={machine.warmTarget}
              caption="Ready for the next job when a slot is free"
              readOnly={!admin}
            />
          </div>
        </Panel>
        <div className="stack">
          <MachineHealth machine={machine} />
          <MachineCache machine={machine} canClear={admin} />
          <Panel title="Cost">
            <div className="spread">
              <div className="stack-tight">
                <p className="strong num">{monthly(hardware.monthlyCost)}</p>
                <p className="caption">The savings report uses this cost.</p>
              </div>
              {admin ? (
                <Action icon={Pencil} size="sm">
                  Edit
                </Action>
              ) : null}
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
      <MachineSlots
        slots={machine.slots}
        usable={defaultLabelJobs(machine.hardware).atOnce}
      />
    </>
  );
}
