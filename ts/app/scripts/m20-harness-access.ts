import type { IncomingMessage } from "node:http";

import type { components } from "../../api/index.ts";
import { activation } from "./fixtures-core.ts";
import { readJson } from "./harness-api.ts";
import { problem, success } from "./harness-state.ts";
import { repositoryInput, roleInput } from "./m20-harness-input.ts";
import type { FixtureReply, M20State } from "./m20-harness-state.ts";

export async function accessReply(
  request: IncomingMessage,
  route: string,
  state: M20State,
): Promise<FixtureReply | undefined> {
  const method = request.method ?? "GET";
  if (route === "members" && method === "GET")
    return { status: 200, body: state.members };
  const memberMatch = /^members\/([^/]+)$/.exec(route);
  if (memberMatch !== null) {
    const member = state.members.members.find(
      (item) => item.id === memberMatch[1],
    );
    if (member === undefined)
      return { status: 404, body: problem("not_found", "Member not found.") };
    if (method === "PUT") {
      const input = roleInput(await readJson(request));
      if (input === undefined)
        return {
          status: 400,
          body: problem("invalid_request", "Choose a role."),
        };
      member.role = input.role;
    } else if (method === "DELETE") {
      state.members.members = state.members.members.filter(
        (item) => item.id !== member.id,
      );
    } else return undefined;
    state.members.open_requests = state.members.members.filter(
      (item) => item.role == null,
    ).length;
    const current = state.session.workspaces[0];
    if (current) {
      if (member.login === state.session.login)
        current.role = method === "DELETE" ? null : member.role;
      current.admins = state.members.members
        .filter((item) => item.role === "admin")
        .flatMap((item) => (item.login ? [item.login] : []));
    }
    return { status: 200, body: success };
  }
  if (route === "organizations" && method === "GET")
    return { status: 200, body: state.organizations };
  if (route === "organizations/add" && method === "POST") {
    const body: components["schemas"]["InstallLink"] = {
      attempt_id: "fixture-m20-attempt",
      url: "/__fixture/github/install?attempt=fixture-m20-attempt",
    };
    return { status: 200, body };
  }
  const match = /^organizations\/([^/]+)(?:\/(.+))?$/.exec(route);
  if (match === null) return undefined;
  const found = state.organizations.find((item) => item.id === match[1]);
  if (found === undefined || found.state === "removed")
    return {
      status: 404,
      body: problem("not_found", "Organization not found."),
    };
  if (match[2] === "repositories") {
    const repositories = state.repositories[found.id] ?? [];
    if (method === "GET") return { status: 200, body: repositories };
    if (method !== "PUT") return undefined;
    const input = repositoryInput(await readJson(request));
    if (input === undefined)
      return {
        status: 400,
        body: problem("invalid_request", "Choose the repositories."),
      };
    found.access = input;
    for (const repository of repositories)
      repository.allowed =
        repository.private &&
        (input.mode === "all_private" || input.ids.includes(repository.id));
    return { status: 200, body: success };
  }
  if (match[2] === "enable" && method === "POST") {
    if (!found.activation.can_enable)
      return {
        status: 409,
        body: problem(
          "conflict",
          "This organization still waits for its plan.",
        ),
      };
    found.activation = activation();
    return { status: 200, body: success };
  }
  if (match[2] === undefined && method === "DELETE") {
    if (
      state.organizations.filter((item) => item.state !== "removed").length < 2
    )
      return {
        status: 409,
        body: problem(
          "conflict",
          "Keep the last organization in this workspace.",
        ),
      };
    state.organizations = state.organizations.filter(
      (item) => item.id !== found.id,
    );
    return { status: 200, body: success };
  }
  return undefined;
}
