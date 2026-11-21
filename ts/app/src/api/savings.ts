// The Savings route of the dashboard API: the report of one UTC range,
// through the one API client.

import type { ApiClient } from "./client.ts";
import { routePath, withQuery } from "./route-path.ts";
import { parseSavings } from "./savings-parse.ts";
import type { SavingsRange } from "./savings-types.ts";

export function createSavingsEndpoints(api: ApiClient) {
  return {
    report: (workspaceId: string, range: SavingsRange) =>
      api.read(
        withQuery(
          routePath("/workspaces/{workspace_id}/savings", {
            workspace_id: workspaceId,
          }),
          { range },
        ),
        parseSavings,
      ),
  };
}

export type SavingsEndpoints = ReturnType<typeof createSavingsEndpoints>;
