// The overflow summary of one workspace, which every role reads for the
// Overflow panel of Machines and for where jobs go while a machine takes
// none. It is a page read: a failed first read shows with Try again, and a
// failed refresh keeps the last summary and shows the failure with Try again.
// Neither hides the machines. The summary reads again only when a page asks,
// never on an interval, because each read asks the providers for the costs
// of the open starts.

import type { ApiResult } from "../api/client.ts";
import type { MachineEndpoints } from "../api/machine-endpoints.ts";
import type { OverflowSettings } from "../api/machine-schema.ts";
import {
  createPageRead,
  type PageRead,
  type PageSnapshot,
} from "./page-store.ts";

export type OverflowSnapshot = PageSnapshot<OverflowSettings>;

export type OverflowStore = PageRead<OverflowSettings>;

/** The summary that a snapshot holds, or null while none is known. */
export function overflowSummary(
  snapshot: OverflowSnapshot,
): OverflowSettings | null {
  return snapshot.status === "ready" ? snapshot.page : null;
}

export function createOverflowStore(options: {
  endpoints: MachineEndpoints;
  workspaceId: string;
  report: <T>(result: ApiResult<T>) => ApiResult<T>;
}): OverflowStore {
  const { endpoints, workspaceId, report } = options;
  return createPageRead({
    read: () => endpoints.overflowSummary(workspaceId),
    report,
  });
}
