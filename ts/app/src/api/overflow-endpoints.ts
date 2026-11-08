// The Boat routes of Setup and Settings under Overflow. The key check saves
// nothing: it sends only the key and returns the wallets that the key can
// use. A connection sends the generated request of its caller as it is: the
// key, the limit, and the chosen wallet.

import type { ApiClient } from "./client.ts";
import { workspacePath } from "./endpoints.ts";
import { parseBoatWallets, parseOverflowSettings } from "./parse-overflow.ts";
import { parseSuccess } from "./parse-session.ts";
import type { BoatConnection, BoatKeyCheck, OverflowPolicy } from "./schema.ts";

/**
 * A connection request that names its wallet. The API connects the personal
 * wallet when a request names none, so each connection sends the wallet that
 * the customer chose.
 */
export type BoatConnectionWithWallet = BoatConnection &
  Required<Pick<BoatConnection, "wallet">>;

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
    checkBoatKey: (workspaceId: string, apiKey: string) => {
      const check: BoatKeyCheck = { api_key: apiKey };
      return client.change(
        "POST",
        workspacePath(workspaceId, "/overflow/wallets"),
        parseBoatWallets,
        check,
      );
    },
    connectBoat: (workspaceId: string, request: BoatConnectionWithWallet) =>
      client.change("POST", connection(workspaceId), parseSuccess, request),
    disconnectBoat: (workspaceId: string) =>
      client.change("DELETE", connection(workspaceId), parseSuccess),
  };
}

export type OverflowEndpoints = ReturnType<typeof createOverflowEndpoints>;
