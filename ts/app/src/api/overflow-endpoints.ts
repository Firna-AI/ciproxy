// The Boat routes of Setup and Settings under Overflow. The connection body
// is built here and nowhere else: the key and the limit. It sends no wallet,
// so the API connects the personal wallet of the Boat account.

import type { ApiClient } from "./client.ts";
import { workspacePath } from "./endpoints.ts";
import { parseOverflowSettings } from "./parse-overflow.ts";
import { parseSuccess } from "./parse-session.ts";
import type { BoatConnection, OverflowPolicy } from "./schema.ts";

/** What a connection sends: a key, and a limit in integer US cents. */
export type BoatKeyInput = { apiKey: string; limitCents: string };

export function boatConnection(input: BoatKeyInput): BoatConnection {
  return { api_key: input.apiKey, spend_limit_cents: input.limitCents };
}

export function createOverflowEndpoints(client: ApiClient) {
  const overflow = (workspaceId: string) =>
    workspacePath(workspaceId, "/overflow");
  const connection = (workspaceId: string) =>
    workspacePath(workspaceId, "/overflow/connection");
  return {
    overflow: (workspaceId: string) =>
      client.read(overflow(workspaceId), parseOverflowSettings),
    saveOverflow: (workspaceId: string, policy: OverflowPolicy) =>
      client.change("PUT", overflow(workspaceId), parseSuccess, policy),
    connectBoat: (workspaceId: string, input: BoatKeyInput) =>
      client.change(
        "POST",
        connection(workspaceId),
        parseSuccess,
        boatConnection(input),
      ),
    disconnectBoat: (workspaceId: string) =>
      client.change("DELETE", connection(workspaceId), parseSuccess),
  };
}

export type OverflowEndpoints = ReturnType<typeof createOverflowEndpoints>;
