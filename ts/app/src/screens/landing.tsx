// The landing at `/`. It reads the session and opens the page that the
// server state asks for. `/?workspace=<id>` opens that workspace: Setup for
// an Admin until it is done, Jobs otherwise. The API sends every sign-in and
// install return here.

import { useEffect } from "react";
import { useLocalSearchParams } from "expo-router";

import type { WorkspaceView } from "../api/schema.ts";
import { SessionWaiting } from "../components/page-state.tsx";
import { sessionLanding, workspaceLanding } from "../routing/landing.ts";
import { idParam, workspaceHref } from "../routing/pages.ts";
import { useServices } from "../services/context.tsx";
import { useSession } from "../services/hooks.ts";

export function LandingScreen() {
  const services = useServices();
  const params = useLocalSearchParams<{ workspace?: string | string[] }>();
  const requested = idParam(params.workspace);
  const snapshot = useSession(`home:${requested ?? ""}`);

  useEffect(() => {
    if (snapshot.status !== "signed-in") return;
    const session = snapshot.session;
    let cancelled = false;
    const go = (href: string) => {
      if (cancelled) return;
      services.clearInstallStarted();
      services.navigator.go(href, { replace: true });
    };
    const open = async (workspace: WorkspaceView) => {
      if (workspace.role !== "admin") {
        go(workspaceLanding(workspace, null));
        return;
      }
      const store = services.setup(workspace.id);
      await store.ensure();
      const setup = store.state.get();
      go(
        setup.status === "ready"
          ? workspaceLanding(workspace, setup.setup)
          : workspaceHref("setup", workspace.id),
      );
    };
    const chosen =
      session.state === "ready"
        ? session.workspaces.find((workspace) => workspace.id === requested)
        : undefined;
    if (chosen !== undefined) {
      void open(chosen);
    } else {
      const landing = sessionLanding(session, {
        installStarted: services.installStarted(),
        preferredWorkspaceId: requested ?? services.lastWorkspace(),
      });
      if ("href" in landing) go(landing.href);
      else void open(landing.workspace);
    }
    return () => {
      cancelled = true;
    };
  }, [snapshot, requested, services]);

  return <SessionWaiting snapshot={snapshot} />;
}
