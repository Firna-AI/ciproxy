// Machines on its one static route: the list, one machine, or Add a machine,
// as the query asks. Each of them is a page of its own for the session check,
// so a change between them checks the access again. The links between them
// push, because a link to the same route with a new query would otherwise
// replace the history entry, and Back would skip the page.

import { useLocalSearchParams } from "expo-router";

import {
  machinesRoute,
  type MachinesRoute,
} from "../../view/machines-routes.ts";
import { WorkspacePageView } from "../workspace-page.tsx";
import { AddMachine } from "./add-machine.tsx";
import { MachineDetail } from "./machine-detail.tsx";
import { MachinesList } from "./machines-list.tsx";

function visitKey(route: MachinesRoute): string {
  return route.kind === "machine" ? `machine:${route.machineId}` : route.kind;
}

export function MachinesScreen() {
  const params = useLocalSearchParams<{
    view?: string | string[];
    machine?: string | string[];
  }>();
  const route = machinesRoute(params.view, params.machine);
  return (
    <WorkspacePageView page="machines" visitKey={visitKey(route)}>
      {(context) => {
        switch (route.kind) {
          case "add":
            return <AddMachine context={context} />;
          case "machine":
            return (
              <MachineDetail
                key={route.machineId}
                context={context}
                machineId={route.machineId}
              />
            );
          case "list":
            return <MachinesList context={context} />;
        }
      }}
    </WorkspacePageView>
  );
}
