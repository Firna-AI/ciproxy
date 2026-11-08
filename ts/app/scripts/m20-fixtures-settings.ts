import type { components } from "../../api/index.ts";
import { activation, workspaceId } from "./fixtures-core.ts";
import { organization } from "./fixtures-setup.ts";

type Schemas = components["schemas"];
type Label = Schemas["LabelView"];

export const organizationIds = {
  primary: "40000000-0000-4000-8000-000000000001",
  second: "40000000-0000-4000-8000-000000000002",
  removed: "40000000-0000-4000-8000-000000000003",
  inactive: "40000000-0000-4000-8000-000000000004",
} as const;

export const labelIds = {
  default: "60000000-0000-4000-8000-000000000001",
  medium: "60000000-0000-4000-8000-000000000002",
  large: "60000000-0000-4000-8000-000000000003",
  custom: "60000000-0000-4000-8000-000000000004",
} as const;

export function fixtureFit(): Schemas["LabelFit"] {
  return {
    eligible_machines: 3,
    fitting_machines: 2,
    minimum_jobs: "1",
    maximum_jobs: "6",
    no_machine_fits: false,
    unavailable_machines: 1,
  };
}

export function fixtureLabels(): Label[] {
  const base: Schemas["LabelInput"] = {
    name: "ciproxy",
    vcpus: 2,
    memory_gb: 8,
    disk_gb: 20,
    allow_machines: true,
    allow_boat: true,
    when_nothing_fits: "wait",
  };
  return [
    { id: labelIds.default, label: base, fit: fixtureFit() },
    {
      id: labelIds.medium,
      label: { ...base, name: "ciproxy-4vcpu", vcpus: 4, memory_gb: 16 },
      fit: { ...fixtureFit(), maximum_jobs: "3" },
    },
    {
      id: labelIds.large,
      label: {
        ...base,
        name: "ciproxy-8vcpu",
        vcpus: 8,
        memory_gb: 32,
        disk_gb: 60,
      },
      fit: { ...fixtureFit(), fitting_machines: 1, maximum_jobs: "1" },
    },
    {
      id: labelIds.custom,
      label: {
        ...base,
        name: "integration-tests",
        vcpus: 2,
        memory_gb: 8,
        disk_gb: 1,
      },
      fit: fixtureFit(),
    },
  ] satisfies Label[];
}

export function fixtureOrganizations(): Schemas["OrganizationView"][] {
  return [
    organization(),
    organization({
      id: organizationIds.second,
      github_id: "8002",
      installation_id: "9002",
      login: "northstar-engineering",
      access: { mode: "all_private" },
      manage_url:
        "https://github.com/organizations/northstar-engineering/settings/installations/9002",
    }),
    organization({
      id: organizationIds.removed,
      github_id: "8003",
      installation_id: "9003",
      login: "northstar-archive",
      state: "removed",
      delete_after: "2026-12-02T10:00:00Z",
      manage_url:
        "https://github.com/organizations/northstar-archive/settings/installations/9003",
    }),
  ] satisfies Schemas["OrganizationView"][];
}

export function inactiveOrganization(
  billing: boolean,
): Schemas["OrganizationView"] {
  return organization({
    id: organizationIds.inactive,
    github_id: "8004",
    installation_id: "9004",
    login: "northstar-research",
    manage_url:
      "https://github.com/organizations/northstar-research/settings/installations/9004",
    activation: activation({
      admitted: false,
      reasons: billing
        ? ["billing_upgrade", "admin_approval"]
        : ["admin_approval"],
      can_enable: !billing,
      added_by: "former-admin",
      upgrade: billing ? { tier: "team", workspace_id: workspaceId } : null,
    }),
  });
}

export function fixtureMembers(): Schemas["MembersView"] {
  return {
    admin_notice:
      "An Admin can add machines and connect Boat. These run jobs and their secrets for every organization in this workspace.",
    open_requests: 1,
    members: [
      {
        id: "101",
        login: "mira",
        role: "admin",
        requested_at: "2026-09-01T09:00:00Z",
      },
      {
        id: "102",
        login: "sasha",
        role: "admin",
        requested_at: "2026-09-02T09:00:00Z",
      },
      {
        id: "103",
        login: "nora",
        role: "billing",
        requested_at: "2026-10-01T09:00:00Z",
      },
      {
        id: "104",
        login: "ellis",
        role: "viewer",
        requested_at: "2026-10-02T09:00:00Z",
      },
      {
        id: "105",
        login: "rowan",
        role: null,
        requested_at: "2026-11-06T10:00:00Z",
      },
    ],
  } satisfies Schemas["MembersView"];
}

export function fixtureRepositories(
  login = "northstar-labs",
): Schemas["RepositoryView"][] {
  return [
    { id: "7001", full_name: `${login}/control`, private: true, allowed: true },
    { id: "7002", full_name: `${login}/runner`, private: true, allowed: false },
    {
      id: "7003",
      full_name: `${login}/public-docs`,
      private: false,
      allowed: false,
    },
  ] satisfies Schemas["RepositoryView"][];
}
