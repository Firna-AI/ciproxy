// The services of one document: one API client, one build gate, one
// navigator, one session, and one Setup and install-command store for each
// workspace. A full-document navigation drops them all, so nothing private
// outlives a sign-out.

import {
  createApiClient,
  type ApiClient,
  type ApiResult,
  type Transport,
} from "../api/client.ts";
import { createEndpoints, type Endpoints } from "../api/endpoints.ts";
import type { InstallLink } from "../api/schema.ts";
import { createBuildGate, type BuildGate } from "../state/build-gate.ts";
import {
  createInstallCommandStore,
  type InstallCommandStore,
} from "../state/install-command.ts";
import {
  createNavigator,
  type DocumentLocation,
  type Navigator,
  type PageRouter,
} from "../state/navigation.ts";
import type { Timers, Visibility } from "../state/poller.ts";
import { createSessionStore, type SessionStore } from "../state/session.ts";
import { createSetupStore, type SetupStore } from "../state/setup.ts";
import { storageKeys, type KeyValueStorage } from "../state/storage.ts";

export type AppEnvironment = {
  transport: Transport;
  location: DocumentLocation;
  router: PageRouter;
  /** Storage that outlives the tab, such as the end of the sign-in. */
  local: KeyValueStorage;
  /** Storage of this tab only. */
  tab: KeyValueStorage;
  now: () => number;
  timers: Timers;
  visibility: Visibility;
  /** The current page: its path and query. */
  page?: () => string;
};

export type AppServices = {
  readonly env: AppEnvironment;
  readonly gate: BuildGate;
  readonly navigator: Navigator;
  readonly client: ApiClient;
  readonly endpoints: Endpoints;
  readonly session: SessionStore;
  setup(workspaceId: string): SetupStore;
  installCommand(workspaceId: string): InstallCommandStore;
  /** Save installation evidence before leaving for GitHub. */
  startInstallation(): Promise<ApiResult<InstallLink>>;
  /** Remember that this tab follows an install link to GitHub. */
  markInstallStarted(): void;
  /** Whether this tab came back from an install that it started. */
  installStarted(): boolean;
  /** Forget the install of this tab, once the landing used it. */
  clearInstallStarted(): void;
  rememberWorkspace(workspaceId: string): void;
  lastWorkspace(): string | null;
  /** The last verified installation that the landing handled, if stored. */
  handledInstallation(): string | null;
  rememberInstallation(installationId: string): void;
  /** The key of the last ended continuation that the browser handled. */
  handledContinuation(): string | null;
  rememberContinuation(key: string): void;
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

export function createAppServices(env: AppEnvironment): AppServices {
  const gate = createBuildGate(env.page);
  let session: SessionStore | null = null;
  const client = createApiClient({
    transport: env.transport,
    gate,
    csrf: () => session?.csrf() ?? null,
    reload: () => env.location.reload(),
  });
  const endpoints = createEndpoints(client);
  const sessionStore = createSessionStore(() => endpoints, env.local, env.now);
  session = sessionStore;
  const navigator = createNavigator(gate, env.location, env.router);

  return {
    env,
    gate,
    navigator,
    client,
    endpoints,
    session: sessionStore,
    setup: cached((workspaceId) =>
      createSetupStore({
        endpoints,
        workspaceId,
        report: sessionStore.report,
        timers: env.timers,
        visibility: env.visibility,
      }),
    ),
    installCommand: cached((workspaceId) =>
      createInstallCommandStore({
        endpoints,
        workspaceId,
        mutate: sessionStore.mutate,
        now: env.now,
      }),
    ),
    async startInstallation() {
      const result = await sessionStore.mutate(() =>
        endpoints.startInstallation(),
      );
      if (result.ok) {
        env.tab.set(storageKeys.installStarted, "1");
        navigator.leave(result.value.url);
      }
      return result;
    },
    markInstallStarted() {
      env.tab.set(storageKeys.installStarted, "1");
    },
    installStarted: () => env.tab.get(storageKeys.installStarted) === "1",
    clearInstallStarted() {
      env.tab.remove(storageKeys.installStarted);
    },
    rememberWorkspace(workspaceId) {
      env.local.set(storageKeys.lastWorkspace, workspaceId);
    },
    lastWorkspace: () => env.local.get(storageKeys.lastWorkspace),
    handledInstallation: () => env.local.get(storageKeys.handledInstallation),
    rememberInstallation(installationId) {
      env.local.set(storageKeys.handledInstallation, installationId);
    },
    handledContinuation: () => env.local.get(storageKeys.handledContinuation),
    rememberContinuation(key) {
      env.local.set(storageKeys.handledContinuation, key);
    },
  };
}
