// One own machine: its platform, slots and warm runners, health, cache, and
// the monthly cost that the savings report uses. A machine that cannot start
// VMs connects with no slots, so it cannot run jobs, and the page names the
// fix: KVM on a Linux machine, a newer macOS on a Mac. A Mac runs at most 2
// job VMs at a time, so its free slots past that limit take no job, and a Mac
// with room for 1 slot says why. A machine without
// room for a slot says why too. A machine whose disk holds fewer jobs than
// its slots says so, as design call 34 says. After Clear cache, or a drain
// that waits for the machine to connect again, a notice says so. A part of
// the cache that the machine did not measure shows Unknown, with no notice.
// Every role sees the machine; only an Admin drains, removes, or changes it.
// The warm runner count is the saved count in every state, also while the
// machine drains, is offline, or has no slots; the slots show the warm
// runners that are ready. A machine that gets its first job image takes no
// job yet, so its slots show no slot bar and no free slot, and a notice says
// that jobs start on it when it is ready. The scene of each state gives the
// machine and its Clear cache confirmation.

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
import type { Machine } from "../../fixtures/machines.ts";
import type { Role } from "../../fixtures/members.ts";
import { macosVmLimit, platformName } from "../../fixtures/platform.ts";
import { defaultLabelJobs } from "../../fixtures/settings.ts";
import { sizePhrase } from "../../fixtures/sizes.ts";
import { ax42OneSlotJobs, macSlotJobs } from "../../fixtures/slots.ts";
import { MachineCache } from "./machine-cache.tsx";
import { MachineHealth } from "./machine-health.tsx";
import { MachineNotice, type NoticeState } from "./machine-notices.tsx";
import { machineScene, type MachineState } from "./machine-scenes.tsx";
import { MachineSlots } from "./machine-slots.tsx";
import "./machines.css";

/** A fourth machine, also a Mac, passes the three machines of Team. */
function overMachines(state: MachineState): boolean {
  return (
    state === "disk-limit" ||
    state === "no-room" ||
    state === "preparing" ||
    state.startsWith("mac")
  );
}

/** The notice that leads the page, or null for a state without one. */
function noticeOf(state: MachineState): NoticeState | null {
  switch (state) {
    case "healthy":
    case "clear-cache":
    case "agent-behind":
    case "cache-unknown":
    case "docker-unknown":
    case "both-unknown":
    case "clear-cache-unknown":
    case "clear-docker-unknown":
    case "clear-both-unknown":
    case "mac":
      return null;
    default:
      return state;
  }
}

function HeaderActions({ state }: { state: MachineState }) {
  if (state === "offline") return <Action tone="danger">Remove machine</Action>;
  if (state === "draining" || state === "saved-offline") {
    return <Action>Stop draining</Action>;
  }
  if (
    state === "no-slots" ||
    state === "no-room" ||
    state === "mac-no-virtualization"
  ) {
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
  const scene = machineScene(state);
  const { machine } = scene;
  const { hardware } = machine;
  const noSlots = machine.slots.length === 0;
  const admin = role === "admin";
  const notice = noticeOf(state);
  return (
    <AppShell
      viewport={viewport}
      tier="team"
      active="machines"
      role={role}
      banner={
        overMachines(state)
          ? usageBanner("over-machines", "other", canPay(role))
          : undefined
      }
    >
      <PageHeader
        crumbs={[{ label: "Machines", to: paths.machines }]}
        title={machine.name}
        status={<MachineStatusLabel status={machine.status} />}
        subtitle={dotJoin(
          platformName[hardware.platform],
          hardware.cpu,
          hardwareSize(hardware),
          hardware.disk,
        )}
        actions={admin ? <HeaderActions state={state} /> : undefined}
      />
      {notice === null ? null : (
        <MachineNotice state={notice} machine={machine} admin={admin} />
      )}
      <div className="grid-2">
        <Panel
          title="Slots"
          action={<TextLink to={paths.jobsHistory}>Job history</TextLink>}
        >
          <div className="stack">
            <SlotsBody machine={machine} />
            {noSlots ? null : <SlotsCaption machine={machine} />}
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
          <MachineCache
            machine={machine}
            canClear={admin}
            confirm={scene.clearCache?.screen}
          />
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
      {scene.overlay}
    </AppShell>
  );
}

/** The slots, their size, and on a Mac the limit of its VMs. */
function SlotsCaption({ machine }: { machine: Machine }) {
  const count = machine.slots.length;
  const slots = `${count} ${count === 1 ? "slot" : "slots"}`;
  const mac = machine.hardware.platform === "macos";
  return (
    <p className="caption">
      {slots} of {sizePhrase(slotSize)}
      {mac ? `, and at most ${macosVmLimit} VMs at once` : ""}. A larger VM size
      takes several slots.
    </p>
  );
}

/** The slot bar and what each slot runs, or why the machine shows none. */
function SlotsBody({ machine }: { machine: Machine }) {
  if (machine.status === "offline") {
    return (
      <p className="small muted">Slots show again when the machine is back.</p>
    );
  }
  if (machine.status === "preparing") {
    return (
      <p className="small muted">Slots take jobs when the machine is ready.</p>
    );
  }
  const mac = machine.hardware.platform === "macos";
  if (machine.slots.length === 0) {
    return (
      <p className="small muted">
        {mac
          ? "No slots. Jobs of macOS labels wait until a Mac has a free slot."
          : "No slots. Jobs run on your other machines and on Boat."}
      </p>
    );
  }
  return (
    <>
      <SlotStrip slots={machine.slots} />
      <MachineSlots
        slots={machine.slots}
        usable={defaultLabelJobs(machine.hardware).atOnce}
        jobs={mac ? macSlotJobs : ax42OneSlotJobs}
        vmLimit={mac ? macosVmLimit : undefined}
      />
    </>
  );
}
