// Readers for the session, its workspaces and setup outcome, and the shared
// Problem and Success bodies.

import {
  array,
  boolean,
  object,
  oneOf,
  optional,
  parser,
  ParseError,
  string,
  time,
} from "./parse-core.ts";
import { readLink } from "./parse-links.ts";
import type {
  PendingInstallation,
  Problem,
  ProblemCode,
  RemovedOrganization,
  SessionView,
  SetupOutcome,
  SignInState,
  Success,
  WorkspaceRole,
  WorkspaceView,
} from "./schema.ts";

const signInState = oneOf<SignInState>({
  ready: true,
  install_required: true,
  app_removed: true,
  access_ended: true,
});

export const workspaceRole = oneOf<WorkspaceRole>({
  admin: true,
  billing: true,
  viewer: true,
});

const problemCode = oneOf<ProblemCode>({
  boat_key_rejected: true,
  sign_in_required: true,
  access_denied: true,
  not_found: true,
  conflict: true,
  review_required: true,
  invalid_request: true,
  github_label_refused: true,
  unavailable: true,
  internal: true,
  method_not_allowed: true,
});

function workspace(value: unknown): WorkspaceView {
  const fields = object(value, "workspace");
  return {
    admins: array(fields.admins, string),
    can_claim_admin: boolean(fields.can_claim_admin),
    id: string(fields.id),
    name: string(fields.name),
    organization_limit_exceeded: boolean(fields.organization_limit_exceeded),
    role: optional(fields.role, workspaceRole),
  };
}

function pending(value: unknown): PendingInstallation {
  const fields = object(value, "install request");
  return {
    created_at: time(fields.created_at),
    github_id: string(fields.github_id),
    id: string(fields.id),
    organization: string(fields.organization),
  };
}

function removed(value: unknown): RemovedOrganization {
  const fields = object(value, "removed organization");
  return {
    delete_after: time(fields.delete_after),
    github_id: string(fields.github_id),
    login: string(fields.login),
  };
}

function setup(value: unknown): SetupOutcome {
  const fields = object(value, "setup outcome");
  switch (fields.kind) {
    case "waiting_approval":
      return { kind: "waiting_approval" };
    case "organization":
      return {
        kind: "organization",
        github_id: string(fields.github_id),
        installation_id: string(fields.installation_id),
        login: string(fields.login),
        suspended: boolean(fields.suspended),
      };
    case "organizations_only":
      return {
        kind: "organizations_only",
        installation_id: string(fields.installation_id),
        remove_url: string(fields.remove_url),
      };
    default:
      throw new ParseError("setup outcome");
  }
}

export const parseSession = parser((value): SessionView => {
  const fields = object(value, "session");
  return {
    csrf_token: string(fields.csrf_token),
    expires_at: time(fields.expires_at),
    install_url: string(fields.install_url),
    installation_continuation: array(fields.installation_continuation, string),
    login: string(fields.login),
    organization_links: array(fields.organization_links, readLink),
    pending_install_requests: array(fields.pending_install_requests, pending),
    reauthorization_required: boolean(fields.reauthorization_required),
    reauthorization_url: optional(fields.reauthorization_url, string),
    removed_organization: optional(fields.removed_organization, removed),
    setup: optional(fields.setup, setup),
    state: signInState(fields.state),
    workspaces: array(fields.workspaces, workspace),
  };
});

export const parseProblem = parser((value): Problem => {
  const fields = object(value, "problem");
  return { code: problemCode(fields.code), message: string(fields.message) };
});

export const parseSuccess = parser((value): Success => {
  const fields = object(value, "result");
  return { ok: boolean(fields.ok) };
});
