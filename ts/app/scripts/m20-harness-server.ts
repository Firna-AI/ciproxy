import type { IncomingMessage, ServerResponse } from "node:http";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

import type { components } from "../../api/index.ts";
import { workspaceId } from "./fixtures-core.ts";
import { setup } from "./fixtures-setup.ts";
import { json, readJson } from "./harness-api.ts";
import { startHarness } from "./harness-server.ts";
import { problem, success } from "./harness-state.ts";
import { accessReply } from "./m20-harness-access.ts";
import { machineReply } from "./m20-harness-machines.ts";
import { settingsReply } from "./m20-harness-settings.ts";
import {
  createM20State,
  updateM20State,
  type M20State,
} from "./m20-harness-state.ts";

function fails(state: M20State, method: string, route: string): boolean {
  const scenario = state.scenario;
  if (scenario === "machines-load-failure") return route === "machines";
  if (scenario === "machine-load-failure")
    return /^machines\/[^/]+$/.test(route);
  if (scenario === "settings-load-failure") return route === "settings";
  if (scenario === "members-load-failure") return route === "members";
  if (scenario === "repositories-load-failure")
    return route.endsWith("/repositories");
  if (scenario === "overflow-summary-failure")
    return route === "overflow/summary";
  if (scenario === "install-command-failure") return route === "install-tokens";
  if (scenario === "install-revoke-failure")
    return route.startsWith("install-tokens/");
  if (scenario === "machine-mutation-failure")
    return method !== "GET" && route.startsWith("machines/");
  return (
    scenario === "settings-mutation-failure" &&
    method !== "GET" &&
    route !== "labels/preview"
  );
}

async function serve(
  request: IncomingMessage,
  response: ServerResponse,
  url: URL,
  state: M20State,
): Promise<boolean> {
  const path = url.pathname;
  if (path === "/__fixture/control" && request.method === "POST") {
    const ok = updateM20State(state, await readJson(request));
    json(response, ok ? 200 : 400, { ok }, state.build);
    return true;
  }
  if (path === "/__fixture/state") {
    json(
      response,
      200,
      {
        scenario: state.scenario,
        build: state.build,
        issued: state.issued,
        revoked: state.revoked,
        facts: state.facts,
      },
      state.build,
    );
    return true;
  }
  if (!path.startsWith("/api/")) return false;
  const method = request.method ?? "GET";
  const send = (status: number, body: unknown) => {
    state.facts.push({ method, path, status, kind: "api" });
    json(response, status, body, state.build);
    return true;
  };
  if (state.scenario === "signed-out")
    return send(401, problem("sign_in_required", "Sign in again."));
  if (
    method !== "GET" &&
    request.headers["x-ciproxy-csrf"] !== state.session.csrf_token
  )
    return send(403, problem("access_denied", "Refresh this sign-in."));
  if (path === "/api/session" && method === "GET")
    return send(200, state.session);
  if (path === "/api/auth/sign-out" && method === "POST") {
    state.scenario = "signed-out";
    return send(200, success);
  }
  const match = /^\/api\/workspaces\/([^/]+)\/(.+)$/.exec(path);
  if (match === null)
    return send(404, problem("not_found", "Route not found."));
  const role = state.session.workspaces.find(
    (item) => item.id === match[1],
  )?.role;
  if (match[1] !== workspaceId || role == null)
    return send(403, problem("access_denied", "Access ended."));
  const route = match[2] ?? "";
  if (
    role !== "admin" &&
    !(
      method === "GET" &&
      (/^machines(?:\/[^/]+)?$/.test(route) || route === "overflow/summary")
    )
  )
    return send(403, problem("access_denied", "Admin role required."));
  if (fails(state, method, route))
    return send(
      503,
      problem("unavailable", "This action is unavailable. Try again."),
    );
  if (route === "setup" && method === "GET")
    return send(
      200,
      setup({
        machines: state.machines.length,
        waiting_for_machine: state.machines.length === 0,
        organizations: state.organizations,
        labels: state.labels,
      }),
    );
  if (route === "banner" && method === "GET") {
    const body: components["schemas"]["PageBannerView"] = { banner: null };
    return send(200, body);
  }
  if (route === "overflow/summary" && method === "GET") {
    const unconnected = state.scenario === "overflow-unconnected";
    const body: components["schemas"]["OverflowSettings"] = {
      available: true,
      account: unconnected
        ? null
        : {
            name: "Northstar Boat",
            connected_at: "2026-09-15T09:00:00Z",
            rejected_at: null,
          },
      enabled: !unconnected,
      next_month_at: "2026-12-01T00:00:00Z",
      spend: unconnected
        ? null
        : {
            bounded: true,
            confirmed_cents: "1400",
            unconfirmed_cents: "200",
            settled_bound_cents: "100",
            limit_reached: false,
          },
      spend_limit_cents: unconnected ? null : "5000",
      wait_seconds: 30,
    };
    return send(200, body);
  }
  const reply =
    (await machineReply(request, route, state)) ??
    (await settingsReply(request, route, state)) ??
    (await accessReply(request, route, state));
  return reply
    ? send(reply.status, reply.body)
    : send(404, problem("not_found", "Route not found."));
}

export async function startM20Harness(port: number, root?: string) {
  const state = createM20State();
  return startHarness(port, root, (request, response, url) =>
    serve(request, response, url, state),
  );
}

if (
  process.argv[1] !== undefined &&
  import.meta.url === pathToFileURL(resolve(process.argv[1])).href
) {
  const port = Number(
    process.argv.find((value) => value.startsWith("--port="))?.slice(7) ??
      "4341",
  );
  if (!Number.isInteger(port) || port < 0 || port > 65535)
    throw new Error("Invalid fixture port");
  const harness = await startM20Harness(port);
  process.stdout.write(`Fixture dashboard: ${harness.origin}\n`);
  for (const signal of ["SIGINT", "SIGTERM"] as const)
    process.once(signal, () => {
      void harness.stop().then(() => process.exit(0));
    });
}
