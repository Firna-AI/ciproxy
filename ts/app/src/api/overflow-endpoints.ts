// Workspace overflow policy and changes scoped to a single provider.

import type { ApiClient } from "./client.ts";
import {
  parseBoatWallets,
  parseOverflowSettings,
  parseProviderSuccess,
} from "./parse-overflow.ts";
import { parseSuccess } from "./parse-session.ts";
import { routePath } from "./route-path.ts";
import type {
  BoatConnection,
  BoatKeyCheck,
  OverflowPolicy,
  OverflowProvider,
  ProviderSpendLimit,
} from "./schema.ts";

/** The current Boat form sends the wallet that the customer chose. */
export type BoatConnectionWithWallet = BoatConnection &
  Required<Pick<BoatConnection, "wallet">>;

export function createOverflowEndpoints(client: ApiClient) {
  const overflow = (workspace_id: string) =>
    routePath("/workspaces/{workspace_id}/overflow", { workspace_id });
  const connection = (workspace_id: string, provider: OverflowProvider) =>
    routePath(
      "/workspaces/{workspace_id}/overflow/providers/{provider}/connection",
      {
        workspace_id,
        provider,
      },
    );
  return {
    overflow: (workspaceId: string) =>
      client.read(overflow(workspaceId), parseOverflowSettings),
    saveOverflow: (workspaceId: string, policy: OverflowPolicy) =>
      client.change("PUT", overflow(workspaceId), parseSuccess, policy),
    checkKey: (
      workspace_id: string,
      provider: OverflowProvider,
      apiKey: string,
    ) => {
      const check: BoatKeyCheck = { api_key: apiKey };
      return client.change(
        "POST",
        routePath(
          "/workspaces/{workspace_id}/overflow/providers/{provider}/key-check",
          {
            workspace_id,
            provider,
          },
        ),
        parseBoatWallets,
        check,
      );
    },
    connectProvider: (
      workspaceId: string,
      provider: OverflowProvider,
      request: BoatConnection,
    ) =>
      client.change(
        "POST",
        connection(workspaceId, provider),
        parseProviderSuccess,
        request,
      ),
    changeProviderKey: (
      workspaceId: string,
      provider: OverflowProvider,
      request: BoatConnection,
    ) =>
      client.change(
        "PUT",
        connection(workspaceId, provider),
        parseProviderSuccess,
        request,
      ),
    disconnectProvider: (workspaceId: string, provider: OverflowProvider) =>
      client.change(
        "DELETE",
        connection(workspaceId, provider),
        parseProviderSuccess,
      ),
    saveProviderLimit: (
      workspace_id: string,
      provider: OverflowProvider,
      limit: ProviderSpendLimit,
    ) =>
      client.change(
        "PUT",
        routePath(
          "/workspaces/{workspace_id}/overflow/providers/{provider}/spend-limit",
          {
            workspace_id,
            provider,
          },
        ),
        parseProviderSuccess,
        limit,
      ),
  };
}

export type OverflowEndpoints = ReturnType<typeof createOverflowEndpoints>;
