// The dashboard routes that this milestone uses, each with its reader. Paths
// follow the generated contract; IDs are encoded as path segments.

import type { paths } from "../../../api/index.ts";
import { apiPrefix, type ApiClient } from "./client.ts";
import { parseInstallLink, parseLink, parseLinks } from "./parse-links.ts";
import { parseInstallCommand, parseMachines } from "./parse-machines.ts";
import { parseSession, parseSuccess } from "./parse-session.ts";
import { parseSetup } from "./parse-setup.ts";
import type { WorkspaceChoice } from "./schema.ts";

const authBegin: keyof paths = "/auth/github";

/** The top-level navigation that starts a GitHub sign-in. */
export const githubSignInHref = apiPrefix + authBegin;

function segment(id: string): string {
  return encodeURIComponent(id);
}

export function workspacePath(workspaceId: string, rest: string): string {
  return `/workspaces/${segment(workspaceId)}${rest}`;
}

export function createEndpoints(client: ApiClient) {
  return {
    session: () => client.read("/session", parseSession),
    signOut: () => client.change("POST", "/auth/sign-out", parseSuccess),
    claimAdmin: (workspaceId: string) =>
      client.change(
        "POST",
        workspacePath(workspaceId, "/claim-admin"),
        parseSuccess,
      ),
    /** Refreshes the person's own links with fresh GitHub proof. */
    links: () => client.read("/installation-links", parseLinks),
    /** Reads one saved link from the store, with no GitHub call. */
    link: (claimId: string) =>
      client.read(`/installation-links/${segment(claimId)}`, parseLink),
    chooseWorkspace: (claimId: string, workspaceId: string) => {
      const choice: WorkspaceChoice = { workspace_id: workspaceId };
      return client.change(
        "PUT",
        `/installation-links/${segment(claimId)}/workspace`,
        parseLink,
        choice,
      );
    },
    addOrganization: (workspaceId: string) =>
      client.change(
        "POST",
        workspacePath(workspaceId, "/organizations/add"),
        parseInstallLink,
      ),
    setup: (workspaceId: string) =>
      client.read(workspacePath(workspaceId, "/setup"), parseSetup),
    machines: (workspaceId: string) =>
      client.read(workspacePath(workspaceId, "/machines"), parseMachines),
    issueInstallCommand: (workspaceId: string) =>
      client.change(
        "POST",
        workspacePath(workspaceId, "/install-tokens"),
        parseInstallCommand,
      ),
    revokeInstallCommand: (workspaceId: string, tokenId: string) =>
      client.change(
        "DELETE",
        workspacePath(workspaceId, `/install-tokens/${segment(tokenId)}`),
        parseSuccess,
      ),
  };
}

export type Endpoints = ReturnType<typeof createEndpoints>;
