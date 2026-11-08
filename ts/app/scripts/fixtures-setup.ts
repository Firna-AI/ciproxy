import type {
  InstallCommand,
  MachineView,
  OrganizationView,
  SetupJob,
  SetupView,
} from "../src/api/schema.ts";
import { activation } from "./fixtures-core.ts";

export function machine(overrides: Partial<MachineView> = {}): MachineView {
  return {
    agent_version: "0.1.0",
    cache: null,
    capacity_known: true,
    concurrent_default_jobs: "2",
    disks: [
      { capacity_bytes: "500000000000", free_bytes: "240000000000", path: "/" },
    ],
    draining: false,
    health_old: false,
    hostname: "forge-01",
    id: "30000000-0000-4000-8000-000000000001",
    kvm: true,
    last_contact_at: "2026-11-06T12:00:00Z",
    memory_bytes: "64000000000",
    monthly_cost_cents: null,
    occupancy: [],
    offline: false,
    processor: "Fixture CPU",
    selected_image_disk_bytes: "40000000000",
    slots: 6,
    threads: 12,
    warm_runners: 1,
    ...overrides,
  } satisfies MachineView;
}

export function organization(
  overrides: Partial<OrganizationView> = {},
): OrganizationView {
  return {
    access: { mode: "selected", ids: ["7001"] },
    activation: activation(),
    delete_after: null,
    github_id: "8001",
    id: "40000000-0000-4000-8000-000000000001",
    installation_id: "9001",
    login: "northstar-labs",
    manage_url:
      "https://github.com/organizations/northstar-labs/settings/installations/9001",
    state: "active",
    ...overrides,
  } satisfies OrganizationView;
}

export function job(overrides: Partial<SetupJob> = {}): SetupJob {
  return {
    assigned_at: "2026-11-06T12:01:00Z",
    completed_at: null,
    id: "50000000-0000-4000-8000-000000000001",
    name: "Build and test",
    phase: "started",
    place: "machine",
    repository: "northstar-labs/control",
    result: null,
    started_at: "2026-11-06T12:01:10Z",
    workflow_ref:
      "northstar-labs/control/.github/workflows/ci.yml@refs/heads/main",
    ...overrides,
  } satisfies SetupJob;
}

export function setup(overrides: Partial<SetupView> = {}): SetupView {
  return {
    current_image_disk_bytes: "40000000000",
    first_job: null,
    labels: [
      {
        fit: {
          eligible_machines: 0,
          fitting_machines: 0,
          maximum_jobs: null,
          minimum_jobs: null,
          no_machine_fits: true,
          unavailable_machines: 0,
        },
        id: "60000000-0000-4000-8000-000000000001",
        label: {
          allow_boat: true,
          allow_machines: true,
          disk_gb: 20,
          memory_gb: 8,
          name: "ciproxy",
          vcpus: 4,
          when_nothing_fits: "wait",
        },
      },
    ],
    machines: 0,
    organizations: [organization()],
    waiting_for_machine: true,
    ...overrides,
  } satisfies SetupView;
}

export function installCommand(
  overrides: Partial<InstallCommand> = {},
): InstallCommand {
  return {
    command: "printf 'fixture-only install command'",
    expires_at: "2026-11-06T13:00:00Z",
    id: "fixture-install-token",
    requirements: "Linux x86-64 · KVM · 4 threads · 8 GB RAM · 40 GB free disk",
    ...overrides,
  } satisfies InstallCommand;
}
