// Readers for the Settings routes: the settings of a workspace, its labels
// and their fit, the members with their profiles and the requests for access,
// and the repositories of one organization. Each reader narrows an unknown
// response body to its generated type, or rejects the whole body.

import {
  array,
  boolean,
  integer,
  object,
  optional,
  parser,
  string,
  time,
} from "./parse-core.ts";
import { workspaceRole } from "./parse-session.ts";
import { readLabel, readLabelFit, readOrganization } from "./parse-setup.ts";
import type {
  MemberOrganization,
  MemberView,
  MembersView,
  RepositoryView,
  SettingsView,
} from "./settings-schema.ts";

function memberOrganization(value: unknown): MemberOrganization {
  const fields = object(value, "request organization");
  return { id: string(fields.id), login: string(fields.login) };
}

function readMember(value: unknown): MemberView {
  const fields = object(value, "member");
  return {
    avatar_url: optional(fields.avatar_url, string),
    granted_at: optional(fields.granted_at, time),
    id: string(fields.id),
    login: optional(fields.login, string),
    name: optional(fields.name, string),
    request_organizations: optional(fields.request_organizations, (list) =>
      array(list, memberOrganization),
    ),
    requested_at: time(fields.requested_at),
    role: optional(fields.role, workspaceRole),
  };
}

function readRepository(value: unknown): RepositoryView {
  const fields = object(value, "repository");
  return {
    allowed: boolean(fields.allowed),
    full_name: string(fields.full_name),
    id: string(fields.id),
    private: boolean(fields.private),
  };
}

export const parseSettings = parser((value): SettingsView => {
  const fields = object(value, "settings");
  return {
    can_add_organization: boolean(fields.can_add_organization),
    labels: array(fields.labels, readLabel),
    name: string(fields.name),
    open_requests: integer(fields.open_requests),
    organizations: array(fields.organizations, readOrganization),
  };
});

export const parseLabel = parser(readLabel);

export const parseLabelFit = parser(readLabelFit);

export const parseMembers = parser((value): MembersView => {
  const fields = object(value, "members");
  return {
    admin_notice: string(fields.admin_notice),
    members: array(fields.members, readMember),
    open_requests: integer(fields.open_requests),
  };
});

export const parseRepositories = parser((value) =>
  array(value, readRepository),
);
