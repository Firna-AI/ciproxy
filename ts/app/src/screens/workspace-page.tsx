// A page of one workspace. It reads the workspace from the query, checks the
// session again for each page, and then shows the page in the shell, or the
// page that asks for access when the person has no role. A page with no
// valid workspace goes back to the landing.

import { useEffect, type ReactNode } from "react";
import { useLocalSearchParams } from "expo-router";

import type { BannerScope } from "../api/banner.ts";
import type {
  SessionView,
  WorkspaceRole,
  WorkspaceView,
} from "../api/schema.ts";
import { SessionWaiting } from "../components/page-state.tsx";
import { AppShell } from "../components/shell/app-shell.tsx";
import { idParam, type WorkspacePage } from "../routing/pages.ts";
import { workspaceAccess } from "../routing/workspace.ts";
import { useServices } from "../services/context.tsx";
import { useRedirect, useSession, useValue } from "../services/hooks.ts";
import type { OverflowSnapshot } from "../state/overflow.ts";
import { createStore } from "../state/store.ts";
import type { SetupSnapshot } from "../state/setup.ts";
import { boatProgress } from "../view/boat.ts";
import { setupComplete, setupModel } from "../view/setup-steps.ts";
import type { SetupProgress } from "../view/shell.ts";
import { NoRoleScreen } from "./no-role.tsx";

export type WorkspaceContext = {
  session: SessionView;
  workspace: WorkspaceView;
  role: WorkspaceRole;
};

const noSetup = createStore<SetupSnapshot>({ status: "loading" });
const noOverflow = createStore<OverflowSnapshot>({ status: "loading" });
const noSkip = createStore<string | null>(null);

/**
 * The progress of Setup for the navigation of an Admin. The Boat settings
 * load only while Setup is open, because only its count needs them.
 */
function useSetupProgress(
  workspaceId: string | null,
  login: string | null,
): SetupProgress | null {
  const services = useServices();
  const store = workspaceId === null ? null : services.setup(workspaceId);
  const overflow = workspaceId === null ? null : services.overflow(workspaceId);
  const skip = workspaceId === null ? null : services.boatSkip(workspaceId);
  const snapshot = useValue(store?.state ?? noSetup);
  const boat = useValue(overflow?.state ?? noOverflow);
  const skippedBy = useValue(skip?.state ?? noSkip);
  const open =
    snapshot.status === "ready" &&
    !setupComplete(snapshot.setup, snapshot.machines);
  useEffect(() => {
    void store?.ensure();
  }, [store]);
  useEffect(() => {
    if (open) void overflow?.ensure();
  }, [open, overflow]);
  if (snapshot.status !== "ready") return null;
  const model = setupModel(
    snapshot.setup,
    snapshot.machines,
    false,
    boatProgress(boat, skippedBy, login ?? ""),
  );
  return { done: model.done, total: model.total, complete: model.complete };
}

export function WorkspacePageView({
  page,
  visitKey = "",
  banner,
  bannerScope = "page",
  children,
}: {
  page: WorkspacePage;
  /**
   * The part of the query that makes a new visit of the page, such as a tab.
   * Each new visit checks the session again, as a page change does.
   */
  visitKey?: string;
  /**
   * A banner of the page that takes the place of the one workspace banner of
   * the shell. Undefined keeps the workspace banner.
   */
  banner?: (context: WorkspaceContext) => ReactNode;
  /** The scope of the one banner; the Organizations tab sets its own. */
  bannerScope?: BannerScope;
  children: (context: WorkspaceContext) => ReactNode;
}) {
  const services = useServices();
  const params = useLocalSearchParams<{ workspace?: string | string[] }>();
  const workspaceId = idParam(params.workspace);
  const snapshot = useSession(`${page}:${workspaceId ?? ""}:${visitKey}`);
  const access =
    snapshot.status === "signed-in"
      ? workspaceAccess(snapshot.session, workspaceId, page)
      : null;
  useRedirect(access?.kind === "redirect" ? access.href : null);
  const shownId =
    access !== null && access.kind !== "redirect" ? access.workspace.id : null;
  useEffect(() => {
    if (shownId !== null) services.rememberWorkspace(shownId);
  }, [shownId, services]);
  const adminId =
    access?.kind === "member" && access.role === "admin"
      ? access.workspace.id
      : null;
  const progress = useSetupProgress(
    adminId,
    snapshot.status === "signed-in" ? snapshot.session.login : null,
  );

  if (
    snapshot.status !== "signed-in" ||
    access === null ||
    access.kind === "redirect"
  ) {
    return <SessionWaiting snapshot={snapshot} />;
  }
  if (access.kind === "no-role") {
    return (
      <NoRoleScreen session={snapshot.session} workspace={access.workspace} />
    );
  }
  const context = {
    session: snapshot.session,
    workspace: access.workspace,
    role: access.role,
  };
  return (
    <AppShell
      session={context.session}
      workspace={context.workspace}
      role={context.role}
      page={page}
      setup={progress}
      banner={banner?.(context)}
      bannerScope={bannerScope}
    >
      {children(context)}
    </AppShell>
  );
}
