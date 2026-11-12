// The signed-in frame of every workspace page. It picks the desktop, rail,
// or phone frame by the window, and puts the notice of a failed session
// check, the one banner of the workspace from the API, and a failed menu
// action above the page.

import type { ReactNode } from "react";

import type { BannerScope } from "../../api/banner.ts";
import type {
  SessionView,
  WorkspaceRole,
  WorkspaceView,
} from "../../api/schema.ts";
import type { WorkspacePage } from "../../routing/pages.ts";
import { navItems, type SetupProgress } from "../../view/shell.ts";
import { WorkspaceBanner } from "../billing/workspace-banner.tsx";
import { Notice } from "../notice.tsx";
import { SessionRefreshNotice } from "../session-notice.tsx";
import { useViewport } from "../viewport.ts";
import { DesktopShell } from "./desktop-shell.tsx";
import { MobileShell } from "./mobile-shell.tsx";
import { useShellActions } from "./shell-actions.ts";

export function AppShell({
  session,
  workspace,
  role,
  page,
  setup,
  banner,
  bannerScope = "page",
  children,
}: {
  session: SessionView;
  workspace: WorkspaceView;
  role: WorkspaceRole;
  page: WorkspacePage;
  setup: SetupProgress | null;
  /** A banner of the page that takes the place of the workspace banner. */
  banner?: ReactNode;
  /** The scope of the one banner; the Organizations tab sets its own. */
  bannerScope?: BannerScope;
  children: ReactNode;
}) {
  const viewport = useViewport();
  const actions = useShellActions(workspace.id);
  const props = {
    session,
    workspace,
    role,
    page,
    items: navItems(workspace.id, role, setup),
    actions,
    children: (
      <>
        <SessionRefreshNotice />
        {banner === undefined ? (
          <WorkspaceBanner
            workspace={workspace}
            page={page}
            scope={bannerScope}
          />
        ) : (
          banner
        )}
        {actions.failure ? (
          <Notice tone="danger" title={actions.failure.title}>
            {actions.failure.body}
          </Notice>
        ) : null}
        {children}
      </>
    ),
  };
  return viewport === "mobile" ? (
    <MobileShell {...props} />
  ) : (
    <DesktopShell {...props} rail={viewport === "narrow"} />
  );
}
