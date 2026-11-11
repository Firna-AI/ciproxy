import type { IncomingMessage } from "node:http";

import type { components } from "../../api/index.ts";
import { readJson } from "./harness-api.ts";
import { fields, problem, success } from "./harness-state.ts";
import type { FixtureReply, M20State } from "./m20-harness-state.ts";

type Schemas = components["schemas"];

export async function machineReply(
  request: IncomingMessage,
  route: string,
  state: M20State,
): Promise<FixtureReply | undefined> {
  const method = request.method ?? "GET";
  if (route === "machines" && method === "GET")
    return { status: 200, body: state.machines };
  if (route === "install-tokens" && method === "POST") {
    state.issued += 1;
    const command: Schemas["InstallCommand"] = {
      id: `20000000-0000-4000-8000-${String(state.issued).padStart(12, "0")}`,
      command: `printf 'fixture-only install command ${state.issued}'`,
      expires_at: "2026-11-06T13:00:00Z",
      requirements:
        "x64 Linux with KVM, 4 cores, 16 GB of memory, and at least 250 GB of disk.",
    };
    return { status: 200, body: command };
  }
  if (route.startsWith("install-tokens/") && method === "DELETE") {
    const id = route.slice("install-tokens/".length);
    if (!state.revokedTokens.has(id)) state.revoked += 1;
    state.revokedTokens.add(id);
    return { status: 200, body: success };
  }
  const match = /^machines\/([^/]+)(?:\/(.+))?$/.exec(route);
  if (match === null) return undefined;
  const machine = state.machines.find((item) => item.id === match[1]);
  if (machine === undefined)
    return { status: 404, body: problem("not_found", "Machine not found.") };
  const action = match[2];
  if (action === undefined && method === "GET")
    return { status: 200, body: machine };
  if (action === "warm-runners" && method === "PUT") {
    const body = await readJson(request);
    if (
      !fields(body) ||
      typeof body.warm_runners !== "number" ||
      !Number.isInteger(body.warm_runners) ||
      body.warm_runners < 0
    )
      return {
        status: 400,
        body: problem("invalid_request", "Enter a warm-runner count."),
      };
    const input: Schemas["WarmRunnerTarget"] = {
      warm_runners: body.warm_runners,
    };
    machine.warm_runners = input.warm_runners;
    return { status: 200, body: success };
  }
  if (action === "cost" && method === "PUT") {
    const body = await readJson(request);
    if (
      !fields(body) ||
      typeof body.monthly_cost_cents !== "string" ||
      !/^\d+$/.test(body.monthly_cost_cents)
    )
      return {
        status: 400,
        body: problem("invalid_request", "Enter the monthly cost."),
      };
    const input: Schemas["MachineCostInput"] = {
      monthly_cost_cents: body.monthly_cost_cents,
    };
    machine.monthly_cost_cents = input.monthly_cost_cents;
    return { status: 200, body: success };
  }
  if (action?.startsWith("actions/") && method === "POST") {
    const command = action.slice("actions/".length);
    if (command === "clear-cache" && machine.offline)
      return {
        status: 503,
        body: problem("unavailable", "This machine is offline."),
      };
    if (command === "drain") {
      machine.draining = true;
      machine.occupancy = machine.occupancy.map((slot) =>
        slot.state === "warm"
          ? { index: slot.index, state: "free", runner_id: null, job_id: null }
          : slot,
      );
    } else if (command === "resume") machine.draining = false;
    else if (command === "remove")
      state.machines = state.machines.filter((item) => item.id !== machine.id);
    else if (command === "clear-cache") {
      if (machine.cache) machine.cache.used_bytes = "0";
    } else return undefined;
    const body: Schemas["CommandResult"] = { connected: !machine.offline };
    return { status: 200, body };
  }
  return undefined;
}
