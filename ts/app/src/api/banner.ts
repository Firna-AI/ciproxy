// The route of the one page banner. The Organizations tab asks for its own
// scope, in which an inactive organization comes before a soft limit.

import { parsePageBannerView } from "./banner-reader.ts";
import type { ApiClient } from "./client.ts";
import { routePath, withQuery } from "./route-path.ts";

/** Every workspace page, or the Organizations tab of Settings. */
export type BannerScope = "page" | "organizations";

export function createBannerEndpoint(api: ApiClient) {
  return {
    banner: (workspaceId: string, scope: BannerScope = "page") =>
      api.read(
        withQuery(
          routePath("/workspaces/{workspace_id}/banner", {
            workspace_id: workspaceId,
          }),
          { page: scope === "organizations" ? "organizations" : undefined },
        ),
        parsePageBannerView,
      ),
  };
}

export type BannerEndpoint = ReturnType<typeof createBannerEndpoint>;
