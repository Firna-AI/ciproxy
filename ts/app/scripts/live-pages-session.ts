import { createApiClient } from "../src/api/client.ts";
import { createEndpoints } from "../src/api/endpoints.ts";
import { createBuildGate } from "../src/state/build-gate.ts";
import type { LivePagesCredentials } from "./live-pages-config.ts";

export function createLivePagesConnection(config: LivePagesCredentials) {
  let csrf = config.csrf_token;
  let missingBuild = false;
  const client = createApiClient({
    gate: createBuildGate(),
    csrf: () => csrf,
    transport: async (path, init) => {
      const headers = new Headers(init.headers);
      headers.set("Cookie", `__Host-ciproxy=${config.session_cookie}`);
      headers.set("Origin", config.base_url);
      const response = await fetch(config.base_url + path, {
        ...init,
        headers,
        redirect: "error",
      });
      if (!response.headers.get("x-ciproxy-build")) missingBuild = true;
      return response;
    },
  });
  const endpoints = createEndpoints(client);
  return {
    client,
    missingBuild: () => missingBuild,
    async session() {
      const result = await endpoints.session();
      if (!result.ok || result.value.state !== "ready")
        throw new Error("Live workspace access is unavailable.");
      const member = result.value.workspaces.find(
        (item) => item.id === config.workspace_id,
      );
      if (member?.role === undefined || member.role === null)
        throw new Error("A workspace role is required.");
      csrf = result.value.csrf_token;
      return member.role;
    },
  };
}
