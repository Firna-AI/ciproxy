// Readers for the nested facts of an organization: the counts of its latest
// complete repository snapshot, and the person who added it, with the
// profile facts that ciproxy kept. The organization reader in
// parse-setup.ts calls them.

import { decimal, object, optional, string, time } from "./parse-core.ts";
import type { MemberProfile, OrganizationRepositories } from "./schema.ts";

export function readRepositoryCounts(value: unknown): OrganizationRepositories {
  const fields = object(value, "repository counts");
  return {
    allowed_private: decimal(fields.allowed_private),
    observed_at: time(fields.observed_at),
    private: decimal(fields.private),
    visible: decimal(fields.visible),
  };
}

/** A person by GitHub ID, with the profile facts that ciproxy kept. */
export function readProfile(value: unknown): MemberProfile {
  const fields = object(value, "person");
  return {
    avatar_url: optional(fields.avatar_url, string),
    id: string(fields.id),
    login: optional(fields.login, string),
    name: optional(fields.name, string),
  };
}
