// Readers for the requester-owned installation links, their offered
// workspaces, and the local activation of a linked organization.

import {
  array,
  boolean,
  object,
  oneOf,
  optional,
  parser,
  string,
} from "./parse-core.ts";
import type {
  ActivationReason,
  ActivationView,
  EligibleWorkspace,
  InstallLink,
  LinkState,
  OrganizationLinkView,
  OrganizationTier,
  OrganizationUpgrade,
} from "./schema.ts";

const linkState = oneOf<LinkState>({
  confirming_installation: true,
  reauthorization_required: true,
  choose_workspace: true,
  waiting_for_admin: true,
  linking: true,
  linked_inactive: true,
  linked: true,
});

const activationReason = oneOf<ActivationReason>({
  billing_upgrade: true,
  admin_approval: true,
});

const organizationTier = oneOf<OrganizationTier>({ team: true });

function upgrade(value: unknown): OrganizationUpgrade {
  const fields = object(value, "upgrade");
  return {
    tier: organizationTier(fields.tier),
    workspace_id: string(fields.workspace_id),
  };
}

export function readActivation(value: unknown): ActivationView {
  const fields = object(value, "activation");
  return {
    added_by: optional(fields.added_by, string),
    admitted: boolean(fields.admitted),
    ask_admin: boolean(fields.ask_admin),
    can_enable: boolean(fields.can_enable),
    preparing: boolean(fields.preparing),
    reasons: array(fields.reasons, activationReason),
    upgrade: optional(fields.upgrade, upgrade),
  };
}

function eligibleWorkspace(value: unknown): EligibleWorkspace {
  const fields = object(value, "workspace choice");
  return {
    id: string(fields.id),
    name: string(fields.name),
    requires_upgrade: boolean(fields.requires_upgrade),
  };
}

export function readLink(value: unknown): OrganizationLinkView {
  const fields = object(value, "organization link");
  return {
    activation: optional(fields.activation, readActivation),
    id: string(fields.id),
    installation_id: string(fields.installation_id),
    organization: string(fields.organization),
    organization_id: optional(fields.organization_id, string),
    reauthorization_required: boolean(fields.reauthorization_required),
    reauthorization_url: string(fields.reauthorization_url),
    state: linkState(fields.state),
    workspace_id: optional(fields.workspace_id, string),
    workspaces: array(fields.workspaces, eligibleWorkspace),
  };
}

export const parseLink = parser(readLink);

export const parseLinks = parser((value) => array(value, readLink));

export const parseInstallLink = parser((value): InstallLink => {
  const fields = object(value, "install link");
  return {
    attempt_id: string(fields.attempt_id),
    url: string(fields.url),
  };
});
