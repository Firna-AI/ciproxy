// Builders of Settings data for the tests. Each builder returns a value of
// the generated type, with fields that a test can replace.

import type {
  ProviderCatalogSize,
  ProviderLabelFit,
  LabelFit,
  LabelInput,
  LabelMachineFit,
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
import { boatPlace, machinePlace } from "../scripts/m20-fixtures-labels.ts";

const gb = (value: number) => String(value * 1_000_000_000);

/** The largest Boat size of the account: 16 vCPU, 32 GB, a 251 GB disk. */
export const xlarge: ProviderCatalogSize = {
  name: "xlarge",
  size: { vcpus: 16, memory_bytes: gb(32), disk_bytes: gb(251) },
};

/** A machine of 12 slots: 24 vCPU and 96 GB, 3 jobs of 8 vCPU at a time. */
export function offer(
  id: string,
  overrides: Partial<LabelMachineFit> = {},
): LabelMachineFit {
  return {
    id,
    hostname: `forge-${id}`,
    slots: "12",
    vcpus: "24",
    memory_bytes: gb(96),
    job_disk_bytes: gb(645),
    required_root_disk_bytes: gb(122),
    concurrent_jobs: "3",
    ...overrides,
  } satisfies LabelMachineFit;
}

export function providerFit(
  overrides: Partial<ProviderLabelFit> = {},
): ProviderLabelFit {
  return {
    provider: "boat",
    fit: {
      state: "fitting",
      refused: false,
      size: xlarge.size,
      size_name: xlarge.name,
    },
    connected: true,
    eligible_size: xlarge,
    required_root_disk_bytes: gb(122),
    largest_size: xlarge,
    hold: null,
    ...overrides,
  };
}

/** The fit of a size of 8 vCPU, 32 GB, and 60 GB of free disk. */
export function fit(overrides: Partial<LabelFit> = {}): LabelFit {
  const places =
    overrides.fitting_places === undefined
      ? [machinePlace, boatPlace]
      : overrides.fitting_places;
  return {
    providers: [providerFit()],
    eligible_machines: 3,
    fitting_machines: 3,
    machines: [offer("1"), offer("2"), offer("3")],
    maximum_jobs: "3",
    minimum_jobs: "3",
    no_machine_fits: false,
    fitting_places: places,
    routing:
      places && places.length > 0
        ? { kind: "places", places }
        : {
            kind: "waits",
            reason: { kind: places === null ? "unavailable" : "nothing_fits" },
          },
    unavailable_machines: 0,
    ...overrides,
  } satisfies LabelFit;
}

export function labelInput(overrides: Partial<LabelInput> = {}): LabelInput {
  return {
    order: [machinePlace, boatPlace],
    disk_gb: 60,
    memory_gb: 32,
    name: "ciproxy-large",
    vcpus: 8,
    when_nothing_fits: "wait",
    ...overrides,
  } satisfies LabelInput;
}

export function label(
  id: string,
  input: Partial<LabelInput> = {},
  labelFit: Partial<LabelFit> = {},
  jobs: string | null = "0",
): LabelView {
  return {
    id,
    label: labelInput(input),
    fit: fit(labelFit),
    jobs_last_30_days: jobs,
    job_count_since: "2026-10-07T12:00:00Z",
    job_count_until: "2026-11-06T12:00:00Z",
  };
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
    added_by: {
      id: "4783921",
      login: "anasouza",
      name: "Ana Souza",
      avatar_url: null,
    },
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
    new_organization_requires_upgrade: false,
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
  overrides: Partial<MemberView> = {},
): MemberView {
  return {
    id,
    login,
    name: null,
    avatar_url: null,
    role,
    requested_at: requestedAt,
    granted_at: role === null ? null : requestedAt,
    request_organizations: null,
    ...overrides,
  } satisfies MemberView;
}

export function membersView(members: MemberView[]): MembersView {
  return {
    admin_notice:
      "An Admin can add machines and connect Boat. These run jobs and their secrets for every organization in this workspace.",
    members,
    open_requests: members.filter((item) => item.role === null).length,
  } satisfies MembersView;
}
