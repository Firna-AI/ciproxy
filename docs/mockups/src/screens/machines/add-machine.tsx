// Add another own machine with the one install command. A fourth machine on
// Team passes the included nodes; the screen says so and keeps it running.

import type { Viewport } from "@mokly/mokly";

import { LinkAction } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import {
  ConnectedMachine,
  InstallCommand,
  WaitingLine,
} from "../../components/install-command.tsx";
import { PageHeader, Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { AppShell } from "../../components/shell/app-shell.tsx";
import { org } from "../../fixtures/org.ts";

export const addMachineStates = ["waiting", "connected"] as const;
export type AddMachineState = (typeof addMachineStates)[number];

export function AddMachineScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: AddMachineState;
}) {
  return (
    <AppShell viewport={viewport} tier="team" active="machines">
      <PageHeader
        crumbs={[{ label: "Machines", to: paths.machines }]}
        title="Add a machine"
      />
      <Panel>
        <div className="stack add-machine">
          <p className="small">
            Run this command on the new server. It installs the ciproxy agent
            and connects the server to {org.login}.
          </p>
          <InstallCommand />
          {state === "connected" ? (
            <>
              <ConnectedMachine name="hetzner-ax42-4" />
              <Notice
                tone="warning"
                title="This is the 4th own node. Team includes 3."
                actions={
                  <LinkAction to={paths.usageOverNodes} size="sm">
                    Add a node to the plan
                  </LinkAction>
                }
              >
                Jobs run on it now. An extra node costs $20 a month.
              </Notice>
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
