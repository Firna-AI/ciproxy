// Add another own machine with the one install command. A fourth machine on
// Team passes the included machines: the banner of every page says so, and
// the machine runs jobs at once. After Revoke token, the command adds no
// machine, and a new command is one click away. An Admin whom GitHub does not
// let manage self-hosted runners sees who can add a machine instead.

import type { Viewport } from "@mokly/mokly";

import { LinkAction } from "../../components/actions.tsx";
import {
  ConnectedMachine,
  InstallCommand,
  NoRunnerRightNotice,
  RevokedCommand,
  WaitingLine,
} from "../../components/install-command.tsx";
import { PageHeader, Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { AppShell } from "../../components/shell/app-shell.tsx";
import { usageBanner } from "../../components/usage-banner.tsx";
import { adminWithoutRight } from "../../fixtures/members.ts";
import { org } from "../../fixtures/org.ts";
import "./machines.css";

export const addMachineStates = [
  "waiting",
  "connected",
  "revoked",
  "no-runner-right",
] as const;
export type AddMachineState = (typeof addMachineStates)[number];

function AddMachineBody({ state }: { state: AddMachineState }) {
  if (state === "no-runner-right") return <NoRunnerRightNotice />;
  if (state === "revoked") return <RevokedCommand next={paths.addMachine} />;
  return (
    <>
      <p className="small">
        Run this command on the new machine. It installs the ciproxy agent and
        connects the machine to {org.login}.
      </p>
      <InstallCommand revokeTo={paths.addMachineRevoked} />
      {state === "connected" ? (
        <>
          <ConnectedMachine name="hetzner-ax42-4" />
          <div className="row">
            <LinkAction to={paths.machinesOverLimit} tone="primary">
              Open Machines
            </LinkAction>
          </div>
        </>
      ) : null}
      {state === "waiting" ? (
        <WaitingLine>Waiting for the machine to connect…</WaitingLine>
      ) : null}
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
      person={state === "no-runner-right" ? adminWithoutRight : undefined}
      banner={
        state === "connected"
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
