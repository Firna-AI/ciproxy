// The pages of Machines on its one static route: the list, one machine, and
// Add a machine. The query names the workspace and, by its opaque ID, the
// machine, so the static hosting serves a reload of each page. The links to
// the pages of Settings and Jobs take their addresses from those pages.

import { idParam, overflowHref, workspaceHref } from "../routing/pages.ts";
import { jobHref, jobsHref } from "./jobs-links.ts";

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

/** History of Jobs, with the finished jobs of the workspace. */
export function jobHistoryHref(workspaceId: string): string {
  return jobsHref(workspaceId, "history");
}

/**
 * The page of one dashboard job, which the Jobs pages own, by the canonical
 * job ID that the API matched to a busy slot. The job ID of a machine report
 * never makes a link.
 */
export function slotJobHref(workspaceId: string, jobId: string): string {
  return jobHref(workspaceId, jobId);
}
