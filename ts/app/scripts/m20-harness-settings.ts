import type { IncomingMessage } from "node:http";

import type { components } from "../../api/index.ts";
import { readJson } from "./harness-api.ts";
import { fields, problem, success } from "./harness-state.ts";
import { fixtureFit } from "./m20-fixtures-settings.ts";
import { labelInput } from "./m20-harness-input.ts";
import type { FixtureReply, M20State } from "./m20-harness-state.ts";

type Schemas = components["schemas"];

function preview(state: M20State): Schemas["LabelFit"] {
  if (state.scenario === "label-no-fit")
    return {
      eligible_machines: 2,
      fitting_machines: 0,
      minimum_jobs: null,
      maximum_jobs: null,
      unavailable_machines: 0,
      no_machine_fits: true,
    };
  if (state.scenario === "label-partial-fit")
    return {
      ...fixtureFit(),
      fitting_machines: 1,
      minimum_jobs: "1",
      maximum_jobs: "1",
    };
  return fixtureFit();
}

export async function settingsReply(
  request: IncomingMessage,
  route: string,
  state: M20State,
): Promise<FixtureReply | undefined> {
  const method = request.method ?? "GET";
  if (route === "settings" && method === "GET") {
    const body: Schemas["SettingsView"] = {
      name: state.session.workspaces[0]?.name ?? "Northstar",
      labels: state.labels,
      organizations: state.organizations,
      open_requests: state.members.open_requests,
      can_add_organization: true,
    };
    return { status: 200, body };
  }
  if (route === "settings" && method === "PUT") {
    const value = await readJson(request);
    if (!fields(value) || typeof value.name !== "string")
      return {
        status: 400,
        body: problem("invalid_request", "Enter a workspace name."),
      };
    const input: Schemas["WorkspaceName"] = { name: value.name };
    const workspace = state.session.workspaces[0];
    if (workspace) workspace.name = input.name;
    return { status: 200, body: success };
  }
  if (route === "labels" && method === "GET")
    return { status: 200, body: state.labels };
  const match = /^labels(?:\/([^/]+))?$/.exec(route);
  if (match === null) return undefined;
  const id = match[1];
  const found = state.labels.find((item) => item.id === id);
  if (id !== undefined && id !== "preview" && found === undefined)
    return { status: 404, body: problem("not_found", "Label not found.") };
  if (method === "DELETE" && found !== undefined) {
    if (found.label.name === "ciproxy")
      return {
        status: 409,
        body: problem("conflict", "Keep the ciproxy label."),
      };
    state.labels = state.labels.filter((item) => item.id !== found.id);
    return { status: 200, body: success };
  }
  if (method !== "POST" && method !== "PUT") return undefined;
  const input = labelInput(await readJson(request));
  if (input === undefined)
    return {
      status: 400,
      body: problem("invalid_request", "Enter the label size."),
    };
  if (id === "preview" && method === "POST")
    return { status: 200, body: preview(state) };
  if (state.scenario === "label-refused" && method === "POST")
    return {
      status: 422,
      body: problem(
        "github_label_refused",
        "GitHub refused this label name. Choose another name.",
      ),
    };
  if (found !== undefined && found.label.name !== input.name)
    return {
      status: 409,
      body: problem("conflict", "A label keeps its name."),
    };
  if (found !== undefined) {
    found.label = input;
    found.fit = preview(state);
    return { status: 200, body: found };
  }
  if (id !== undefined || method !== "POST") return undefined;
  state.labelSequence += 1;
  const body: Schemas["LabelView"] = {
    id: `60000000-0000-4000-8000-${String(state.labelSequence).padStart(12, "0")}`,
    label: input,
    fit: preview(state),
  };
  state.labels.push(body);
  return { status: 200, body };
}
