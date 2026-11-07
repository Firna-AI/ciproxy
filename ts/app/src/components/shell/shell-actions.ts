// The actions of the shell menus: Add an organization, Claim the Admin role,
// and Sign out. Each runs once at a time, and a failure shows in the shell
// until the next action.

import { useCallback, useRef, useState } from "react";

import { pages, workspaceHomeHref } from "../../routing/pages.ts";
import { useServices } from "../../services/context.tsx";
import { claimFailure, type FailureCopy } from "../../view/claim.ts";

export type ShellAction = "add-organization" | "claim-admin" | "sign-out";

export type ShellFailure = FailureCopy;

const addFailure: ShellFailure = {
  title: "Unable to start the install on GitHub",
  body: "Try Add an organization again.",
};

const signOutFailure: ShellFailure = {
  title: "Unable to sign out",
  body: "Try Sign out again.",
};

export function useShellActions(workspaceId: string) {
  const services = useServices();
  const running = useRef<ShellAction | null>(null);
  const [busy, setBusy] = useState<ShellAction | null>(null);
  const [failure, setFailure] = useState<ShellFailure | null>(null);

  const run = useCallback(
    async (action: ShellAction, work: () => Promise<ShellFailure | null>) => {
      if (running.current !== null) return;
      running.current = action;
      setBusy(action);
      setFailure(null);
      const next = await work();
      running.current = null;
      setBusy(null);
      setFailure(next);
    },
    [],
  );

  const addOrganization = useCallback(
    () =>
      run("add-organization", async () => {
        const result = await services.session.mutate(() =>
          services.endpoints.addOrganization(workspaceId),
        );
        if (!result.ok) return addFailure;
        services.markInstallStarted();
        services.navigator.leave(result.value.url);
        return null;
      }),
    [run, services, workspaceId],
  );

  const claimAdmin = useCallback(
    () =>
      run("claim-admin", async () => {
        const result = await services.session.claimAdmin(workspaceId);
        if (!result.ok) return claimFailure(result.failure);
        services.navigator.go(workspaceHomeHref(workspaceId));
        return null;
      }),
    [run, services, workspaceId],
  );

  const signOut = useCallback(
    () =>
      run("sign-out", async () => {
        const result = await services.session.signOut();
        if (!result.ok) return signOutFailure;
        services.navigator.leave(pages.signIn);
        return null;
      }),
    [run, services],
  );

  return { busy, failure, addOrganization, claimAdmin, signOut };
}
