// Builders of Settings data for the tests. Each builder returns a value of
// the generated type, with fields that a test can replace.

import type {
  LabelFit,
  LabelInput,
  LabelView,
  OrganizationView,
} from "../src/api/schema.ts";
import type {
  MemberView,
  MembersView,
  SettingsView,
} from "../src/api/settings-schema.ts";
import { activation } from "../scripts/fixtures-core.ts";
import { organization } from "../scripts/fixtures-setup.ts";

export function fit(overrides: Partial<LabelFit> = {}): LabelFit {
  return {
    eligible_machines: 3,
    fitting_machines: 3,
    maximum_jobs: "3",
    minimum_jobs: "3",
    no_machine_fits: false,
    unavailable_machines: 0,
    ...overrides,
  } satisfies LabelFit;
}

export function labelInput(overrides: Partial<LabelInput> = {}): LabelInput {
  return {
    allow_boat: true,
    allow_machines: true,
    disk_gb: 60,
    memory_gb: 32,
    name: "ciproxy-8vcpu",
    vcpus: 8,
    when_nothing_fits: "wait",
    ...overrides,
  } satisfies LabelInput;
}

export function label(
  id: string,
  input: Partial<LabelInput> = {},
  labelFit: Partial<LabelFit> = {},
): LabelView {
  return { id, label: labelInput(input), fit: fit(labelFit) };
}

/** A linked organization by login, with a stable ID from its number. */
export function linked(
  number: number,
  login: string,
  overrides: Partial<OrganizationView> = {},
): OrganizationView {
  return organization({
    id: `40000000-0000-4000-8000-00000000000${number}`,
    github_id: `800${number}`,
    installation_id: `900${number}`,
    login,
    ...overrides,
  });
}

/** An organization whose owner removed the App on GitHub. */
export function removed(
  number: number,
  login: string,
  deleteAfter: string,
): OrganizationView {
  return linked(number, login, { state: "removed", delete_after: deleteAfter });
}

/** An inactive organization with its holds, as an Admin reads it. */
export function held(
  number: number,
  login: string,
  reasons: OrganizationView["activation"]["reasons"],
  workspaceId: string,
): OrganizationView {
  const billing = reasons.includes("billing_upgrade");
  return linked(number, login, {
    activation: activation({
      admitted: false,
      reasons,
      added_by: "4783921",
      can_enable: !billing && reasons.includes("admin_approval"),
      upgrade: billing ? { tier: "team", workspace_id: workspaceId } : null,
    }),
  });
}

export function settingsView(
  overrides: Partial<SettingsView> = {},
): SettingsView {
  return {
    can_add_organization: true,
    labels: [
      label("60000000-0000-4000-8000-000000000001", { name: "ciproxy" }),
    ],
    name: "Northstar",
    open_requests: 0,
    organizations: [linked(1, "northstar-labs")],
    ...overrides,
  } satisfies SettingsView;
}

export function member(
  id: string,
  login: string | null,
  role: MemberView["role"],
  requestedAt = "2026-09-01T09:00:00Z",
): MemberView {
  return { id, login, role, requested_at: requestedAt } satisfies MemberView;
}

export function membersView(members: MemberView[]): MembersView {
  return {
    admin_notice:
      "An Admin can add machines and connect Boat. These run jobs and their secrets for every organization in this workspace.",
    members,
    open_requests: members.filter((item) => item.role === null).length,
  } satisfies MembersView;
}
