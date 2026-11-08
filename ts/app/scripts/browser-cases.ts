import { workspaceId } from "./fixtures-core.ts";
import type { ScenarioName } from "./harness-scenarios.ts";
import { reviewCases } from "./browser-review-cases.ts";

export type SmokeCase = {
  name: string;
  scenario: ScenarioName;
  path: string;
  heading: string;
  text?: string;
  storage?: Readonly<Record<string, string>>;
};

const jobs = `/jobs?workspace=${workspaceId}`;
const setup = `/setup?workspace=${workspaceId}`;
const add = "/add-organization";

export const smokeCases: readonly SmokeCase[] = [
  ...reviewCases,
  {
    name: "sign-in",
    scenario: "signed-out",
    path: "/sign-in",
    heading: "Run GitHub Actions on your own machines",
  },
  {
    name: "auth-failed",
    scenario: "signed-out",
    path: "/sign-in?reason=github",
    heading: "Run GitHub Actions on your own machines",
    text: "GitHub sign-in did not finish",
  },
  {
    name: "auth-expired",
    scenario: "signed-out",
    path: jobs,
    heading: "Run GitHub Actions on your own machines",
    text: "Your sign-in ended after 30 days",
  },
  {
    name: "install",
    scenario: "install",
    path: "/install",
    heading: "Install ciproxy on GitHub",
  },
  {
    name: "install-pending",
    scenario: "install-pending",
    path: "/install",
    heading: "Install ciproxy on GitHub",
    text: "Waiting for an owner of orbit-labs",
  },
  {
    name: "install-reauthorize",
    scenario: "install-reauthorize",
    path: "/install",
    heading: "Your workspace is ready",
  },
  {
    name: "personal",
    scenario: "personal",
    path: "/personal-account",
    heading: "ciproxy works with GitHub organizations",
  },
  {
    name: "app-removed",
    scenario: "app-removed",
    path: jobs,
    heading: "The GitHub App was removed from northstar-labs",
  },
  {
    name: "access-ended",
    scenario: "access-ended",
    path: jobs,
    heading: "Your access ended",
  },
  {
    name: "no-role",
    scenario: "no-role",
    path: jobs,
    heading: "Waiting for an Admin of Northstar",
  },
  {
    name: "no-role-claim",
    scenario: "no-role-claim",
    path: jobs,
    heading: "Waiting for an Admin of Northstar",
  },
  { name: "admin", scenario: "admin", path: jobs, heading: "Jobs" },
  { name: "billing", scenario: "billing", path: jobs, heading: "Jobs" },
  { name: "viewer", scenario: "viewer", path: jobs, heading: "Jobs" },
  {
    name: "billing-claim",
    scenario: "billing-claim",
    path: jobs,
    heading: "Jobs",
  },
  {
    name: "viewer-claim",
    scenario: "viewer-claim",
    path: jobs,
    heading: "Jobs",
  },
  {
    name: "soft-limit",
    scenario: "soft-limit",
    path: jobs,
    heading: "Jobs",
    text: "Free includes 1 organization",
  },
  {
    name: "links-picker",
    scenario: "links-picker",
    path: add,
    heading: "Choose a workspace for northstar-labs",
  },
  {
    name: "links-request",
    scenario: "links-request",
    path: add,
    heading: "Waiting for an owner of orbit-labs",
  },
  {
    name: "links-recovery",
    scenario: "links-recovery",
    path: add,
    heading: "Add an organization",
    text: "Unable to confirm the install on GitHub",
  },
  {
    name: "links-waiting",
    scenario: "links-waiting",
    path: add,
    heading: "northstar-labs waits for a workspace",
  },
  {
    name: "links-confirming",
    scenario: "links-confirming",
    path: add,
    heading: "Confirming the install on northstar-labs",
  },
  {
    name: "links-linking",
    scenario: "links-linking",
    path: add,
    heading: "Adding northstar-labs to Northstar",
  },
  {
    name: "links-inactive-plan",
    scenario: "links-inactive-plan",
    path: add,
    heading: "Choose a workspace for northstar-labs",
  },
  {
    name: "links-inactive-admin",
    scenario: "links-inactive-admin",
    path: add,
    heading: "Choose a workspace for northstar-labs",
  },
  {
    name: "links-inactive-both",
    scenario: "links-inactive-both",
    path: add,
    heading: "Choose a workspace for northstar-labs",
  },
  {
    name: "links-preparing",
    scenario: "links-preparing",
    path: add,
    heading: "Choose a workspace for northstar-labs",
  },
  {
    name: "setup-waiting",
    scenario: "setup-waiting",
    path: setup,
    heading: "Set up ciproxy",
    text: "Waiting for the machine to connect…",
  },
  {
    name: "setup-connected",
    scenario: "setup-waiting",
    path: setup,
    heading: "Set up ciproxy",
  },
  {
    name: "setup-no-kvm",
    scenario: "setup-no-kvm",
    path: setup,
    heading: "Set up ciproxy",
    text: "forge-01 cannot start VMs",
  },
  {
    name: "setup-available",
    scenario: "setup-available",
    path: setup,
    heading: "Set up ciproxy",
    text: "Build and test in northstar-labs/control is waiting for a runner…",
  },
  {
    name: "setup-assigned",
    scenario: "setup-assigned",
    path: setup,
    heading: "Set up ciproxy",
    text: "Build and test in northstar-labs/control is starting…",
  },
  {
    name: "setup-started",
    scenario: "setup-started",
    path: setup,
    heading: "Set up ciproxy",
    text: "Build and test in northstar-labs/control is running on your machine…",
  },
  {
    name: "setup-success",
    scenario: "setup-success",
    path: setup,
    heading: "Set up ciproxy",
    text: "Your first job passed",
  },
  {
    name: "setup-failure",
    scenario: "setup-failure",
    path: setup,
    heading: "Set up ciproxy",
    text: "Your first job finished",
  },
  {
    name: "setup-load-failure",
    scenario: "setup-load-failure",
    path: setup,
    heading: "Set up ciproxy",
    text: "Unable to load Setup",
  },
  {
    name: "command-load-failure",
    scenario: "command-load-failure",
    path: setup,
    heading: "Set up ciproxy",
    text: "Unable to create the install command",
  },
  {
    name: "command-revoke-failure",
    scenario: "command-revoke-failure",
    path: setup,
    heading: "Set up ciproxy",
  },
  {
    name: "command-expired",
    scenario: "command-expired",
    path: setup,
    heading: "Set up ciproxy",
    text: "The install command ended",
  },
];
