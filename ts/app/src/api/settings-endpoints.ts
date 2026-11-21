// The Settings routes of the dashboard, beside the common endpoints: the
// settings of a workspace and its name, the labels and their fit preview,
// the repository access, take-out, and Turn on of each organization, and the
// members with their roles. Each path is a route of the generated contract,
// so a renamed route fails the type check, and IDs travel as path segments.
// Add an organization stays in the common endpoints, with the shell.

import type { paths } from "../../../api/index.ts";
import type { ApiClient } from "./client.ts";
import { parseSuccess } from "./parse-session.ts";
import type {
  LabelInput,
  RepositorySelection,
  WorkspaceRole,
} from "./schema.ts";
import {
  parseLabel,
  parseLabelFit,
  parseMembers,
  parseRepositories,
  parseSettings,
} from "./settings-parse.ts";
import type { RoleChange, WorkspaceName } from "./settings-schema.ts";

type Route = keyof paths;

/** The names of the path parameters of a route, such as "workspace_id". */
type Params<R extends string> =
  R extends `${string}{${infer Name}}${infer Rest}`
    ? Name | Params<Rest>
    : never;

/** A route of the contract with each path parameter encoded in place. */
export function routePath<R extends Route>(
  template: R,
  params: Readonly<Record<Params<R>, string>>,
): string {
  const values: Readonly<Record<string, string>> = params;
  return template.replace(/\{(\w+)\}/g, (_match, name: string) =>
    encodeURIComponent(values[name] ?? ""),
  );
}

export function createSettingsEndpoints(client: ApiClient) {
  const workspace = (workspace_id: string) => ({ workspace_id });
  const label = (workspace_id: string, label_id: string) => ({
    workspace_id,
    label_id,
  });
  const organization = (workspace_id: string, organization_id: string) => ({
    workspace_id,
    organization_id,
  });
  const member = (workspace_id: string, member_id: string) => ({
    workspace_id,
    member_id,
  });
  return {
    settings: (workspaceId: string) =>
      client.read(
        routePath(
          "/workspaces/{workspace_id}/settings",
          workspace(workspaceId),
        ),
        parseSettings,
      ),
    rename: (workspaceId: string, name: string) => {
      const body: WorkspaceName = { name };
      return client.change(
        "PUT",
        routePath(
          "/workspaces/{workspace_id}/settings",
          workspace(workspaceId),
        ),
        parseSuccess,
        body,
      );
    },
    previewLabel: (workspaceId: string, input: LabelInput) =>
      client.change(
        "POST",
        routePath(
          "/workspaces/{workspace_id}/labels/preview",
          workspace(workspaceId),
        ),
        parseLabelFit,
        input,
      ),
    createLabel: (workspaceId: string, input: LabelInput) =>
      client.change(
        "POST",
        routePath("/workspaces/{workspace_id}/labels", workspace(workspaceId)),
        parseLabel,
        input,
      ),
    updateLabel: (workspaceId: string, labelId: string, input: LabelInput) =>
      client.change(
        "PUT",
        routePath(
          "/workspaces/{workspace_id}/labels/{label_id}",
          label(workspaceId, labelId),
        ),
        parseLabel,
        input,
      ),
    deleteLabel: (workspaceId: string, labelId: string) =>
      client.change(
        "DELETE",
        routePath(
          "/workspaces/{workspace_id}/labels/{label_id}",
          label(workspaceId, labelId),
        ),
        parseSuccess,
      ),
    repositories: (workspaceId: string, organizationId: string) =>
      client.read(
        routePath(
          "/workspaces/{workspace_id}/organizations/{organization_id}/repositories",
          organization(workspaceId, organizationId),
        ),
        parseRepositories,
      ),
    saveRepositories: (
      workspaceId: string,
      organizationId: string,
      selection: RepositorySelection,
    ) =>
      client.change(
        "PUT",
        routePath(
          "/workspaces/{workspace_id}/organizations/{organization_id}/repositories",
          organization(workspaceId, organizationId),
        ),
        parseSuccess,
        selection,
      ),
    takeOut: (workspaceId: string, organizationId: string) =>
      client.change(
        "DELETE",
        routePath(
          "/workspaces/{workspace_id}/organizations/{organization_id}",
          organization(workspaceId, organizationId),
        ),
        parseSuccess,
      ),
    turnOn: (workspaceId: string, organizationId: string) =>
      client.change(
        "POST",
        routePath(
          "/workspaces/{workspace_id}/organizations/{organization_id}/enable",
          organization(workspaceId, organizationId),
        ),
        parseSuccess,
      ),
    members: (workspaceId: string) =>
      client.read(
        routePath("/workspaces/{workspace_id}/members", workspace(workspaceId)),
        parseMembers,
      ),
    saveRole: (workspaceId: string, memberId: string, role: WorkspaceRole) => {
      const body: RoleChange = { role };
      return client.change(
        "PUT",
        routePath(
          "/workspaces/{workspace_id}/members/{member_id}",
          member(workspaceId, memberId),
        ),
        parseSuccess,
        body,
      );
    },
    removeMember: (workspaceId: string, memberId: string) =>
      client.change(
        "DELETE",
        routePath(
          "/workspaces/{workspace_id}/members/{member_id}",
          member(workspaceId, memberId),
        ),
        parseSuccess,
      ),
  };
}

export type SettingsEndpoints = ReturnType<typeof createSettingsEndpoints>;
