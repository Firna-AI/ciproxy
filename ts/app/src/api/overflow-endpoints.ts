// The Boat routes of Setup and Settings under Overflow. The key check and
// the connection body are built here and nowhere else. The key check saves
// nothing: it sends only the key and returns the wallets that the key can
// use. The connection sends the key, the limit, and the chosen wallet.

import type { ApiClient } from "./client.ts";
import { workspacePath } from "./endpoints.ts";
import { parseBoatWallets, parseOverflowSettings } from "./parse-overflow.ts";
import { parseSuccess } from "./parse-session.ts";
import type { BoatConnection, BoatKeyCheck, OverflowPolicy } from "./schema.ts";

/** What a connection sends: a key, a limit in US cents, and a wallet. */
export type BoatKeyInput = {
  apiKey: string;
  limitCents: string;
  /** The wallet identity that the key check offered and the person chose. */
  wallet: string;
};

export function boatConnection(input: BoatKeyInput): BoatConnection {
  return {
    api_key: input.apiKey,
    spend_limit_cents: input.limitCents,
    wallet: input.wallet,
  };
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
    checkBoatKey: (workspaceId: string, apiKey: string) => {
      const check: BoatKeyCheck = { api_key: apiKey };
      return client.change(
        "POST",
        workspacePath(workspaceId, "/overflow/wallets"),
        parseBoatWallets,
        check,
      );
    },
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
