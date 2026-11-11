// The pages of Machines on its one static route: the list, one machine, and
// Add a machine. The query names the workspace and, by its opaque ID, the
// machine, so the static hosting serves a reload of each page.

import { idParam, overflowHref, workspaceHref } from "../routing/pages.ts";

export type MachinesRoute =
  { kind: "list" } | { kind: "add" } | { kind: "machine"; machineId: string };

type Query = string | readonly string[] | undefined;

/** The page that the query asks for. Add a machine wins over a machine. */
export function machinesRoute(view: Query, machine: Query): MachinesRoute {
  if (view === "add") return { kind: "add" };
  const machineId = idParam(machine);
  return machineId === null ? { kind: "list" } : { kind: "machine", machineId };
}

export function machinesHref(workspaceId: string): string {
  return workspaceHref("machines", workspaceId);
}

export function machineHref(workspaceId: string, machineId: string): string {
  return `${machinesHref(workspaceId)}&machine=${encodeURIComponent(machineId)}`;
}

export function addMachineHref(workspaceId: string): string {
  return `${machinesHref(workspaceId)}&view=add`;
}

/** Settings under Overflow, a page that Settings owns. */
export function overflowSettingsHref(workspaceId: string): string {
  return overflowHref(workspaceId);
}

export function jobHistoryHref(workspaceId: string): string {
  return workspaceHref("jobs", workspaceId);
}

/**
 * The page of one dashboard job, which the Jobs pages own: `/job` with the
 * workspace and the canonical job ID that the API matched to a busy slot.
 * The job ID of a machine report never makes a link.
 */
export function slotJobHref(workspaceId: string, jobId: string): string {
  const query = new URLSearchParams({ workspace: workspaceId, job: jobId });
  return `/job?${query.toString()}`;
}
