// The machine routes of the dashboard, beside the common endpoints: the
// machines of a workspace, the commands of one machine, its warm runner
// target and monthly cost, and the overflow summary that every role reads.
// Paths follow the generated contract, and IDs travel as path segments.
// The install command routes stay in the common endpoints, with the store
// that Setup shares.

import type { ApiClient } from "./client.ts";
import { parseCommandResult, parseOverflowSummary } from "./machine-parse.ts";
import type {
  MachineCommand,
  MachineCostInput,
  WarmRunnerTarget,
} from "./machine-schema.ts";
import { parseMachines } from "./parse-machines.ts";
import { parseSuccess } from "./parse-session.ts";

function workspacePath(workspaceId: string, rest: string): string {
  return `/workspaces/${encodeURIComponent(workspaceId)}${rest}`;
}

function machinePath(
  workspaceId: string,
  machineId: string,
  rest: string,
): string {
  return workspacePath(
    workspaceId,
    `/machines/${encodeURIComponent(machineId)}${rest}`,
  );
}

export function createMachineEndpoints(client: ApiClient) {
  return {
    machines: (workspaceId: string) =>
      client.read(workspacePath(workspaceId, "/machines"), parseMachines),
    command: (
      workspaceId: string,
      machineId: string,
      command: MachineCommand,
    ) =>
      client.change(
        "POST",
        machinePath(workspaceId, machineId, `/actions/${command}`),
        parseCommandResult,
      ),
    setWarmRunners: (workspaceId: string, machineId: string, count: number) => {
      const target: WarmRunnerTarget = { warm_runners: count };
      return client.change(
        "PUT",
        machinePath(workspaceId, machineId, "/warm-runners"),
        parseSuccess,
        target,
      );
    },
    setMonthlyCost: (workspaceId: string, machineId: string, cents: string) => {
      const cost: MachineCostInput = { monthly_cost_cents: cents };
      return client.change(
        "PUT",
        machinePath(workspaceId, machineId, "/cost"),
        parseSuccess,
        cost,
      );
    },
    overflowSummary: (workspaceId: string) =>
      client.read(
        workspacePath(workspaceId, "/overflow/summary"),
        parseOverflowSummary,
      ),
  };
}

export type MachineEndpoints = ReturnType<typeof createMachineEndpoints>;
