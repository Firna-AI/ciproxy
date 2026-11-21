import { storageKeys } from "../src/state/storage.ts";
import type { SmokeCase } from "./browser-cases.ts";
import {
  longRenamedWorkspaceName,
  longWorkspaceName,
  otherWorkspaceId,
  workspaceId,
} from "./fixtures-core.ts";
import { returnedInstallationId } from "./fixtures-returns.ts";

function memory(
  started: boolean,
  handled?: string,
): Readonly<Record<string, string>> {
  return {
    [storageKeys.lastWorkspace]: otherWorkspaceId,
    ...(started ? { [storageKeys.installStarted]: "1" } : {}),
    ...(handled === undefined
      ? {}
      : { [storageKeys.handledInstallation]: handled }),
  };
}

const jobs = `/jobs?workspace=${workspaceId}`;
const waiting = "Waiting for an Admin of Northstar";
const setup = "Set up ciproxy";

export const reviewCases: readonly SmokeCase[] = [
  {
    name: "workspace-long",
    scenario: "workspace-long",
    path: jobs,
    heading: "Jobs",
  },
  {
    name: "links-long-names-visible",
    scenario: "links-long-names-visible",
    path: "/add-organization",
    heading: `Choose a workspace for ${longWorkspaceName}`,
  },
  {
    name: "links-renamed-open-visible",
    scenario: "links-renamed-open-visible",
    path: "/add-organization",
    heading: `${longWorkspaceName} joined ${longRenamedWorkspaceName}`,
  },
  {
    name: "root-install-completed",
    scenario: "root-install-completed",
    path: "/",
    heading: setup,
    storage: memory(true, "9000"),
  },
  {
    name: "root-install-viewer",
    scenario: "root-install-viewer",
    path: "/",
    heading: setup,
    storage: memory(true, "9000"),
  },
  {
    name: "root-install-github",
    scenario: "root-install-github",
    path: "/",
    heading: setup,
    storage: memory(false),
  },
  {
    name: "root-install-github-empty",
    scenario: "root-install-github",
    path: "/",
    heading: setup,
    storage: memory(false, ""),
  },
  {
    name: "root-install-handled",
    scenario: "root-install-handled",
    path: "/",
    heading: "Jobs",
    storage: memory(false, returnedInstallationId),
  },
  {
    name: "root-explicit-workspace",
    scenario: "root-explicit-workspace",
    path: `/?workspace=${otherWorkspaceId}`,
    heading: "Jobs",
    storage: memory(false, "9000"),
  },
  {
    name: "root-install-progress",
    scenario: "root-install-pending",
    path: "/",
    heading: "Confirming the install on northstar-labs",
    storage: memory(true, "9000"),
  },
  {
    name: "root-removed-old-claim",
    scenario: "root-removed-old-claim",
    path: "/",
    heading: "The GitHub App was removed from northstar-labs",
    storage: memory(false, "9001"),
  },
  {
    name: "root-removed-old-outcome",
    scenario: "root-removed-old-outcome",
    path: "/",
    heading: "The GitHub App was removed from northstar-labs",
    storage: memory(false),
  },
  {
    name: "root-reinstall-open",
    scenario: "root-reinstall-open",
    path: "/",
    heading: "Install ciproxy on GitHub",
    storage: memory(true, "9001"),
  },
  {
    name: "root-access-recovery",
    scenario: "root-access-recovery",
    path: "/",
    heading: "Install ciproxy on GitHub",
    storage: memory(true, "9001"),
  },
  {
    name: "session-refresh-no-role",
    scenario: "session-refresh-no-role",
    path: jobs,
    heading: waiting,
  },
  {
    name: "session-refresh-workspace",
    scenario: "session-refresh-workspace",
    path: jobs,
    heading: "Jobs",
  },
  {
    name: "claim-failure-no-role",
    scenario: "claim-failure-no-role",
    path: jobs,
    heading: waiting,
  },
  {
    name: "claim-failure-workspace",
    scenario: "claim-failure-workspace",
    path: jobs,
    heading: "Jobs",
  },
  {
    name: "claim-failure-csrf",
    scenario: "claim-failure-csrf",
    path: jobs,
    heading: waiting,
  },
  {
    name: "claim-network-failure",
    scenario: "claim-network-failure",
    path: jobs,
    heading: waiting,
  },
];
