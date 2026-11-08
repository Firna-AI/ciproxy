import type { IncomingMessage, ServerResponse } from "node:http";

import type { InstallLink, WorkspaceChoice } from "../src/api/schema.ts";
import { activation } from "./fixtures-core.ts";
import { installCommand } from "./fixtures-setup.ts";
import { recordedScenario } from "./harness-scenarios.ts";
import {
  fields,
  problem,
  success,
  type HarnessState,
} from "./harness-state.ts";

export async function readJson(request: IncomingMessage): Promise<unknown> {
  let body = "";
  for await (const chunk of request) {
    body += String(chunk);
    if (body.length > 16_384) return undefined;
  }
  try {
    return JSON.parse(body);
  } catch {
    return undefined;
  }
}

export function json(
  response: ServerResponse,
  status: number,
  body: unknown,
  build: string,
): void {
  response.writeHead(status, {
    "Content-Type": "application/json",
    "Cache-Control": "no-store",
    "x-ciproxy-build": build,
  });
  response.end(JSON.stringify(body));
}

function choice(value: unknown): WorkspaceChoice | undefined {
  return fields(value) && typeof value.workspace_id === "string"
    ? { workspace_id: value.workspace_id }
    : undefined;
}

export async function serveApi(
  request: IncomingMessage,
  response: ServerResponse,
  path: string,
  state: HarnessState,
): Promise<void> {
  const method = request.method ?? "GET";
  const current = state.scenario;
  const send = (status: number, body: unknown) => {
    state.facts.push({ method, path, status, kind: "api" });
    json(response, status, body, state.build);
  };
  if (
    path === "/api/auth/github" ||
    path === "/api/auth/github/callback" ||
    path === "/api/github/setup"
  ) {
    const outcome =
      path === "/api/auth/github" ? current.signInOutcome : undefined;
    if (outcome !== undefined) state.scenario = recordedScenario(outcome);
    state.facts.push({ method, path, status: 302, kind: "navigation" });
    response.writeHead(302, {
      Location: outcome === undefined ? "/__fixture/navigation-complete" : "/",
      "x-ciproxy-build": state.build,
    });
    response.end();
    return;
  }
  if (current.signedOut) {
    send(401, problem("sign_in_required", "Sign in again."));
    return;
  }
  if (
    method !== "GET" &&
    request.headers["x-ciproxy-csrf"] !== current.session.csrf_token
  ) {
    send(403, problem("access_denied", "Refresh this sign-in."));
    return;
  }
  if (path === "/api/session") {
    state.sessionReads += 1;
    if (current.name.startsWith("session-refresh-") && state.sessionReads > 1) {
      send(503, problem("unavailable", "The access check is unavailable."));
      return;
    }
    send(200, current.session);
    return;
  }
  if (path === "/api/auth/sign-out" && method === "POST") {
    current.signedOut = true;
    send(200, success);
    return;
  }
  if (path === "/api/installation-links/start" && method === "POST") {
    send(200, {
      attempt_id: "fixture-attempt",
      url: "/__fixture/github/install?attempt=fixture-attempt",
    } satisfies InstallLink);
    return;
  }
  if (path === "/api/installation-links" && method === "GET") {
    send(200, current.session.organization_links);
    return;
  }
  const claim = /^\/api\/installation-links\/([^/]+)(\/workspace)?$/.exec(path);
  if (claim !== null) {
    const found = current.session.organization_links.find(
      (item) => item.id === decodeURIComponent(claim[1] ?? ""),
    );
    if (found === undefined) {
      send(404, problem("not_found", "Link not found."));
      return;
    }
    if (claim[2] === "/workspace" && method === "PUT") {
      const picked = choice(await readJson(request));
      const offered = found.workspaces.find(
        (item) => item.id === picked?.workspace_id,
      );
      if (offered === undefined || found.state !== "choose_workspace") {
        send(409, problem("conflict", "Choose a current workspace."));
        return;
      }
      found.workspace_id = offered.id;
      found.state = offered.requires_upgrade ? "linked_inactive" : "linked";
      found.activation = activation({
        admitted: !offered.requires_upgrade,
        reasons: offered.requires_upgrade ? ["billing_upgrade"] : [],
        upgrade: offered.requires_upgrade
          ? { tier: "team", workspace_id: offered.id }
          : null,
      });
      if (current.linkedOutcome !== undefined)
        Object.assign(found, current.linkedOutcome, {
          workspace_id: offered.id,
        });
    }
    send(200, found);
    return;
  }
  const workspace = /^\/api\/workspaces\/([^/]+)\/(.+)$/.exec(path);
  if (workspace === null) {
    send(404, problem("not_found", "Route not found."));
    return;
  }
  const id = decodeURIComponent(workspace[1] ?? "");
  const member = current.session.workspaces.find((item) => item.id === id);
  if (member === undefined || current.session.state !== "ready") {
    send(403, problem("access_denied", "Access ended."));
    return;
  }
  const action = workspace[2];
  if (action === "claim-admin" && method === "POST") {
    if (current.name === "claim-network-failure") {
      state.facts.push({ method, path, status: 0, kind: "api" });
      request.socket.destroy();
      return;
    }
    if (current.name.startsWith("claim-failure-")) {
      const csrf = current.name === "claim-failure-csrf";
      send(
        csrf ? 403 : 503,
        problem(
          csrf ? "access_denied" : "unavailable",
          "The claim did not finish.",
        ),
      );
      return;
    }
  }
  if (action === "claim-admin" && method === "POST" && member.can_claim_admin) {
    member.role = "admin";
    member.can_claim_admin = false;
    send(200, success);
    return;
  }
  if (member.role !== "admin") {
    send(403, problem("access_denied", "Admin role required."));
    return;
  }
  if (action === "organizations/add" && method === "POST") {
    send(200, {
      attempt_id: "fixture-attempt",
      url: "/__fixture/github/install?attempt=fixture-attempt",
    } satisfies InstallLink);
    return;
  }
  if (action === "setup" && method === "GET") {
    send(
      current.setupFails ? 503 : 200,
      current.setupFails
        ? problem("unavailable", "Setup is unavailable.")
        : current.setup,
    );
    return;
  }
  if (action === "machines" && method === "GET") {
    send(200, current.machines);
    return;
  }
  if (action === "install-tokens" && method === "POST") {
    if (current.commandFails) {
      send(503, problem("unavailable", "Try again."));
      return;
    }
    state.issued += 1;
    send(
      200,
      installCommand({
        id: `fixture-install-${state.issued}`,
        command: `printf 'fixture-only install command ${state.issued}'`,
        expires_at:
          current.name === "command-expired"
            ? "2026-11-06T11:59:00Z"
            : "2026-11-06T13:00:00Z",
      }),
    );
    return;
  }
  if (action?.startsWith("install-tokens/") && method === "DELETE") {
    if (current.revokeFails) {
      send(503, problem("unavailable", "Try again."));
      return;
    }
    state.revoked += 1;
    send(200, success);
    return;
  }
  send(404, problem("not_found", "Route not found."));
}
