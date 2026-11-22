// Add another own machine with the one install command. A fourth machine on
// Team passes the included machines: the banner of every page says so. A new
// machine first gets its job image, and jobs start on it when it is ready:
// hetzner-ax42-4 shows that state. The fourth machine office-mini-1 runs jobs,
// and it reports fewer jobs at a time than its slots, because of its disk.
// After Revoke token, the command adds no machine, and a new command is one
// click away. In a what-if, mac-studio-1 connects with the same command and
// runs the jobs of macOS labels. The Admin role alone adds a machine, as the answer to item 50
// says.

import type { Viewport } from "@mokly/mokly";

import { LinkAction } from "../../components/actions.tsx";
import {
  ConnectedMachine,
  InstallCommand,
  RevokedCommand,
  WaitingLine,
} from "../../components/install-command.tsx";
import { PageHeader, Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { AppShell } from "../../components/shell/app-shell.tsx";
import { usageBanner } from "../../components/usage-banner.tsx";
import { macStudioMachine } from "../../fixtures/macs.ts";
import { officeMini } from "../../fixtures/machines.ts";
import { preparingAx42 } from "../../fixtures/new-machine.ts";
import { workspace } from "../../fixtures/org.ts";
import "./machines.css";

export const addMachineStates = [
  "waiting",
  "preparing",
  "connected",
  "revoked",
  "mac-connected",
] as const;
export type AddMachineState = (typeof addMachineStates)[number];

/** The machine of each state that joined. */
const joined = {
  preparing: preparingAx42,
  connected: officeMini,
  "mac-connected": macStudioMachine,
} as const;

/** The Machines list that shows the machine that joined. */
const joinedList = {
  preparing: paths.machinesPreparing,
  connected: paths.machinesOverLimit,
  "mac-connected": paths.machinesMacs,
} as const;

/** The machine that joined, and the Machines list that shows it. */
function JoinedMachine({ state }: { state: keyof typeof joined }) {
  const preparing = state === "preparing";
  const machine = joined[state];
  return (
    <>
      <ConnectedMachine
        name={machine.name}
        hardware={machine.hardware}
        preparing={preparing}
      />
      <div className="row">
        <LinkAction to={joinedList[state]} tone="primary">
          Open Machines
        </LinkAction>
      </div>
    </>
  );
}

function AddMachineBody({ state }: { state: AddMachineState }) {
  if (state === "revoked") return <RevokedCommand next={paths.addMachine} />;
  return (
    <>
      <p className="small">
        Run this command on the new Linux machine or Mac. It installs the
        ciproxy agent and connects the machine to {workspace.name}.
      </p>
      <InstallCommand revokeTo={paths.addMachineRevoked} />
      {state === "waiting" ? (
        <WaitingLine>Waiting for the machine to connect…</WaitingLine>
      ) : (
        <JoinedMachine state={state} />
      )}
    </>
  );
}

export function AddMachineScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: AddMachineState;
}) {
  return (
    <AppShell
      viewport={viewport}
      tier="team"
      active="machines"
      banner={
        state !== "waiting" && state !== "revoked"
          ? usageBanner("over-machines", "other")
          : undefined
      }
    >
      <PageHeader
        crumbs={[{ label: "Machines", to: paths.machines }]}
        title="Add a machine"
      />
      <Panel>
        <div className="stack add-machine">
          <AddMachineBody state={state} />
        </div>
      </Panel>
    </AppShell>
  );
}
