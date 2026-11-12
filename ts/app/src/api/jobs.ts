// The Jobs routes of the dashboard API: the live queue, the history with its
// filters, and one job. Every route reads through the one API client.

import type { ApiClient } from "./client.ts";
import { parseJobsLive } from "./jobs-live-parse.ts";
import { parseJobDetail, parseJobsHistory } from "./jobs-parse.ts";
import type { HistoryQuery } from "./jobs-types.ts";
import { routePath, withQuery } from "./route-path.ts";

export function createJobsEndpoints(api: ApiClient) {
  return {
    live: (workspaceId: string) =>
      api.read(
        routePath("/workspaces/{workspace_id}/jobs/live", {
          workspace_id: workspaceId,
        }),
        parseJobsLive,
      ),
    /** Finished jobs of the plan window that match the filters. */
    history: (workspaceId: string, query: HistoryQuery) =>
      api.read(
        withQuery(
          routePath("/workspaces/{workspace_id}/jobs/history", {
            workspace_id: workspaceId,
          }),
          query,
        ),
        parseJobsHistory,
      ),
    detail: (workspaceId: string, jobId: string) =>
      api.read(
        routePath("/workspaces/{workspace_id}/jobs/{job_id}", {
          workspace_id: workspaceId,
          job_id: jobId,
        }),
        parseJobDetail,
      ),
  };
}

export type JobsEndpoints = ReturnType<typeof createJobsEndpoints>;
