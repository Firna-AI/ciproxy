import type { components } from "../../api/index.ts";
import { activation, fixtureTime, workspaceId } from "./fixtures-core.ts";
import { organization } from "./fixtures-setup.ts";

type Schemas = components["schemas"];

export const organizationIds = {
  primary: "40000000-0000-4000-8000-000000000001",
  second: "40000000-0000-4000-8000-000000000002",
  removed: "40000000-0000-4000-8000-000000000003",
  inactive: "40000000-0000-4000-8000-000000000004",
} as const;

/** Ana Souza added northstar-research, and she is no longer an Admin. */
const formerAdmin: Schemas["MemberProfile"] = {
  id: "4783921",
  login: "anasouza",
  name: "Ana Souza",
  avatar_url: null,
};

function counts(
  visible: number,
  privateCount: number,
  allowed: number,
): Schemas["OrganizationRepositories"] {
  return {
    visible: String(visible),
    private: String(privateCount),
    allowed_private: String(allowed),
    observed_at: fixtureTime,
  };
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
      joined_at: "2026-11-02T10:00:00Z",
      repositories: counts(3, 2, 2),
      manage_url:
        "https://github.com/organizations/northstar-engineering/settings/installations/9002",
    }),
    organization({
      id: organizationIds.removed,
      github_id: "8003",
      installation_id: "9003",
      login: "northstar-archive",
      state: "removed",
      joined_at: "2026-09-15T09:00:00Z",
      removed_at: "2026-11-02T10:00:00Z",
      delete_after: "2026-12-02T10:00:00Z",
      repositories: null,
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
    joined_at: "2026-11-05T09:00:00Z",
    access: { mode: "all_private" },
    repositories: counts(4, 4, 4),
    added_by: formerAdmin,
    manage_url:
      "https://github.com/organizations/northstar-research/settings/installations/9004",
    activation: activation({
      admitted: false,
      reasons: billing
        ? ["billing_upgrade", "admin_approval"]
        : ["admin_approval"],
      can_enable: !billing,
      added_by: formerAdmin.id,
      upgrade: billing ? { tier: "team", workspace_id: workspaceId } : null,
    }),
  });
}

function member(
  id: string,
  login: string,
  name: string | null,
  role: Schemas["MemberView"]["role"],
  requestedAt: string,
  grantedAt: string | null,
): Schemas["MemberView"] {
  return {
    id,
    login,
    name,
    avatar_url: null,
    role,
    requested_at: requestedAt,
    granted_at: grantedAt,
    request_organizations:
      role === null
        ? [{ id: organizationIds.primary, login: "northstar-labs" }]
        : null,
  };
}

export function fixtureMembers(): Schemas["MembersView"] {
  return {
    admin_notice:
      "An Admin can add machines and connect Boat. These run jobs and their secrets for every organization in this workspace.",
    open_requests: 1,
    members: [
      member(
        "101",
        "mira",
        "Mira Chen",
        "admin",
        "2026-09-01T09:00:00Z",
        "2026-09-01T09:00:00Z",
      ),
      member(
        "102",
        "sasha",
        "Sasha Novak",
        "admin",
        "2026-09-02T09:00:00Z",
        "2026-09-04T15:30:00Z",
      ),
      member("103", "nora", null, "billing", "2026-10-01T09:00:00Z", null),
      member(
        "104",
        "ellis",
        "Ellis Park",
        "viewer",
        "2026-10-02T09:00:00Z",
        "2026-10-03T08:15:00Z",
      ),
      member("105", "rowan", "Rowan Hale", null, "2026-11-06T10:00:00Z", null),
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
