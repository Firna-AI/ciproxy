import type { components } from "../../api/index.ts";
import { session, workspace } from "./fixtures-core.ts";
import { fields, type RequestFact } from "./harness-state.ts";
import { fixtureMachines, machineIds } from "./m20-fixtures-machines.ts";
import {
  fixtureLabels,
  nothingFits,
  withoutBoat,
} from "./m20-fixtures-labels.ts";
import {
  fixtureMembers,
  fixtureOrganizations,
  fixtureRepositories,
  inactiveOrganization,
  organizationIds,
} from "./m20-fixtures-settings.ts";

type Schemas = components["schemas"];

export const m20Scenarios = [
  "machines",
  "machines-empty",
  "machines-long-name",
  "machine-agent-behind",
  "overflow-unconnected",
  "overflow-summary-failure",
  "machines-load-failure",
  "machine-load-failure",
  "machine-mutation-failure",
  "add-machine-waiting",
  "add-machine-connected",
  "install-command-failure",
  "install-revoke-failure",
  "settings",
  "settings-load-failure",
  "settings-mutation-failure",
  "members-load-failure",
  "repositories-load-failure",
  "label-refused",
  "label-no-fit",
  "label-partial-fit",
  "organizations-single-with-removed",
  "organizations-free",
  "organizations-inactive-admin",
  "organizations-inactive-both",
  "organizations-suspended",
  "viewer",
  "billing",
  "no-role",
  "signed-out",
] as const;

export type M20Scenario = (typeof m20Scenarios)[number];
export type FixtureReply = { status: number; body: unknown };
export type M20State = {
  scenario: M20Scenario;
  build: string;
  facts: RequestFact[];
  session: Schemas["SessionView"];
  machines: Schemas["MachineView"][];
  labels: Schemas["LabelView"][];
  organizations: Schemas["OrganizationView"][];
  members: Schemas["MembersView"];
  repositories: Record<string, Schemas["RepositoryView"][]>;
  issued: number;
  revoked: number;
  labelSequence: number;
  revokedTokens: Set<string>;
  /** A new organization would need Team, as on Free. */
  newOrganizationRequiresUpgrade: boolean;
};

export function createM20State(scenario: M20Scenario = "machines"): M20State {
  const organizations = fixtureOrganizations();
  const state: M20State = {
    scenario,
    build: "fixture-build-m20-a",
    facts: [],
    session: session(),
    machines: fixtureMachines(),
    labels: fixtureLabels(),
    organizations,
    members: fixtureMembers(),
    repositories: Object.fromEntries(
      organizations.map((item) => [item.id, fixtureRepositories(item.login)]),
    ),
    issued: 0,
    revoked: 0,
    labelSequence: 4,
    revokedTokens: new Set(),
    newOrganizationRequiresUpgrade: false,
  };
  if (["viewer", "billing", "no-role"].includes(scenario))
    state.session.workspaces = [
      workspace({
        role:
          scenario === "viewer"
            ? "viewer"
            : scenario === "billing"
              ? "billing"
              : null,
      }),
    ];
  if (scenario === "machines-empty" || scenario === "add-machine-waiting")
    state.machines = [];
  if (scenario === "label-no-fit") {
    state.machines = state.machines.filter(
      (item) =>
        item.hostname === "forge-01" || item.hostname === "office-mini-1",
    );
    for (const machine of state.machines) machine.concurrent_default_jobs = "0";
    for (const label of state.labels) {
      label.label = { ...label.label, vcpus: 64, memory_gb: 256, disk_gb: 200 };
      label.fit = nothingFits("266000000000");
    }
  }
  if (scenario === "overflow-unconnected")
    for (const label of state.labels) label.fit = withoutBoat(label.fit);
  if (scenario === "add-machine-connected")
    state.machines = state.machines.filter(
      (item) =>
        item.id === machineIds.primary || item.id === machineIds.diskLimited,
    );
  // forge-01 reports the release before the newest; the slots stay as they are.
  const forge = state.machines.find((item) => item.id === machineIds.primary);
  if (scenario === "machine-agent-behind" && forge) {
    forge.agent_version = "0.4.1";
    forge.agent_up_to_date = false;
  }
  if (scenario === "machines-long-name" && state.machines[0])
    state.machines[0].hostname =
      "northstar-continuous-integration-machine-with-a-long-host-name-01";
  if (scenario === "organizations-single-with-removed")
    state.organizations = state.organizations.filter(
      (item) => item.login !== "northstar-engineering",
    );
  if (scenario.startsWith("organizations-inactive"))
    state.organizations.push(inactiveOrganization(scenario.endsWith("both")));
  // Free runs one organization, so a new one waits for Team; the hold of
  // both reasons waits for Team too.
  if (scenario === "organizations-free")
    state.organizations = state.organizations.filter(
      (item) => item.id === organizationIds.primary,
    );
  if (
    scenario === "organizations-free" ||
    scenario === "organizations-inactive-both"
  )
    state.newOrganizationRequiresUpgrade = true;
  for (const item of state.organizations)
    state.repositories[item.id] ??= fixtureRepositories(item.login);
  if (scenario === "organizations-suspended" && state.organizations[1])
    state.organizations[1].state = "suspended";
  return state;
}

export function updateM20State(state: M20State, value: unknown): boolean {
  if (!fields(value)) return false;
  if (value.scenario !== undefined) {
    const name = m20Scenarios.find((item) => item === value.scenario);
    if (name === undefined) return false;
    const next = createM20State(name);
    if (value.reset === false) {
      next.facts = state.facts;
      next.issued = state.issued;
      next.revoked = state.revoked;
      next.revokedTokens = state.revokedTokens;
      next.build = state.build;
    }
    Object.assign(state, next);
  }
  if (
    typeof value.build === "string" &&
    /^fixture-build-m20-[a-z]+$/.test(value.build)
  )
    state.build = value.build;
  return true;
}
