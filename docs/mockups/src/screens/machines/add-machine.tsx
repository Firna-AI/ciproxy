// Add another own machine with the one install command. A fourth machine on
// Team passes the included machines: the banner of every page says so, and
// the machine runs jobs at once.

import type { Viewport } from "@mokly/mokly";

import { LinkAction } from "../../components/actions.tsx";
import {
  ConnectedMachine,
  InstallCommand,
  WaitingLine,
} from "../../components/install-command.tsx";
import { PageHeader, Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { AppShell } from "../../components/shell/app-shell.tsx";
import { usageBanner } from "../../components/usage-banner.tsx";
import { org } from "../../fixtures/org.ts";
import "./machines.css";

export const addMachineStates = ["waiting", "connected"] as const;
export type AddMachineState = (typeof addMachineStates)[number];

export function AddMachineScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: AddMachineState;
}) {
  const connected = state === "connected";
  return (
    <AppShell
      viewport={viewport}
      tier="team"
      active="machines"
      banner={connected ? usageBanner("over-machines", "other") : undefined}
    >
      <PageHeader
        crumbs={[{ label: "Machines", to: paths.machines }]}
        title="Add a machine"
      />
      <Panel>
        <div className="stack add-machine">
          <p className="small">
            Run this command on the new machine. It installs the ciproxy agent
            and connects the machine to {org.login}.
          </p>
          <InstallCommand />
          {connected ? (
            <>
              <ConnectedMachine name="hetzner-ax42-4" />
              <div className="row">
                <LinkAction to={paths.machinesOverLimit} tone="primary">
                  Open Machines
                </LinkAction>
              </div>
            </>
          ) : (
            <WaitingLine>Waiting for the machine to connect…</WaitingLine>
          )}
        </div>
      </Panel>
    </AppShell>
  );
}
