// The Settings stores of one document, beside the shared services: one
// Settings endpoints object on the shared client, one settings and one
// members store for each workspace, and one repositories store for each
// organization. They live as long as the services of the document, so a
// full-document navigation drops them, as it drops every other store.

import {
  createSettingsEndpoints,
  type SettingsEndpoints,
} from "../api/settings-endpoints.ts";
import type { AppServices } from "../services/services.ts";
import { createMembersStore, type MembersStore } from "./settings-members.ts";
import {
  createRepositoriesStore,
  type RepositoriesStore,
} from "./settings-repositories.ts";
import { createSettingsStore, type SettingsStore } from "./settings-store.ts";

export type SettingsServices = {
  readonly endpoints: SettingsEndpoints;
  settings(workspaceId: string): SettingsStore;
  members(workspaceId: string): MembersStore;
  repositories(workspaceId: string, organizationId: string): RepositoriesStore;
};

function cached<T>(create: (key: string) => T): (key: string) => T {
  const values = new Map<string, T>();
  return (key) => {
    const known = values.get(key);
    if (known !== undefined) return known;
    const value = create(key);
    values.set(key, value);
    return value;
  };
}

function createSettingsServices(app: AppServices): SettingsServices {
  const endpoints = createSettingsEndpoints(app.client);
  const { mutate, report } = app.session;
  const refreshSession = () => app.session.refresh();
  const settings = cached((workspaceId) =>
    createSettingsStore({
      endpoints,
      workspaceId,
      mutate,
      report,
      refreshSession,
    }),
  );
  const members = cached((workspaceId) =>
    createMembersStore({
      endpoints,
      workspaceId,
      mutate,
      report,
      afterChange: () =>
        Promise.all([settings(workspaceId).load(), refreshSession()]),
    }),
  );
  const repositories = cached((workspaceId) =>
    cached((organizationId) =>
      createRepositoriesStore({
        read: () => endpoints.repositories(workspaceId, organizationId),
        report,
      }),
    ),
  );
  return {
    endpoints,
    settings,
    members,
    repositories: (workspaceId, organizationId) =>
      repositories(workspaceId)(organizationId),
  };
}

const registry = new WeakMap<AppServices, SettingsServices>();

/** The Settings stores of the document that `app` serves. */
export function settingsServices(app: AppServices): SettingsServices {
  const known = registry.get(app);
  if (known !== undefined) return known;
  const created = createSettingsServices(app);
  registry.set(app, created);
  return created;
}
