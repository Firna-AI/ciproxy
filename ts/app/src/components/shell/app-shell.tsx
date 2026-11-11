// The signed-in frame of every workspace page. It picks the desktop, rail,
// or phone frame by the window, and puts the notice of a failed session
// check, the banner of the organization soft limit, and a failed menu action
// above the page.

import type { ReactNode } from "react";
import { Link } from "expo-router";
import { Text } from "react-native";

import type {
  SessionView,
  WorkspaceRole,
  WorkspaceView,
} from "../../api/schema.ts";
import type { WorkspacePage } from "../../routing/pages.ts";
import {
  navItems,
  organizationBanner,
  type SetupProgress,
} from "../../view/shell.ts";
import { ActionLink } from "../actions.tsx";
import { Notice } from "../notice.tsx";
import { SessionRefreshNotice } from "../session-notice.tsx";
import { text } from "../text.ts";
import { useViewport } from "../viewport.ts";
import { DesktopShell } from "./desktop-shell.tsx";
import { MobileShell } from "./mobile-shell.tsx";
import { useShellActions } from "./shell-actions.ts";

/** The banner of the organization soft limit, on every page. */
export function SoftLimitBanner({
  workspace,
  role,
}: {
  workspace: WorkspaceView;
  role: WorkspaceRole;
}) {
  const banner = organizationBanner(workspace, role);
  if (banner === null) return null;
  const { action } = banner;
  return (
    <Notice
      tone="warning"
      title={banner.title}
      actions={
        "ask" in action ? (
          <Text style={text.caption}>{action.ask}</Text>
        ) : (
          <Link href={action.href} asChild>
            <ActionLink label={action.label} href={action.href} size="sm" />
          </Link>
        )
      }
    >
      {banner.body}
    </Notice>
  );
}

export function AppShell({
  session,
  workspace,
  role,
  page,
  setup,
  banner,
  children,
}: {
  session: SessionView;
  workspace: WorkspaceView;
  role: WorkspaceRole;
  page: WorkspacePage;
  setup: SetupProgress | null;
  /** A banner of the page that takes the place of the soft-limit banner. */
  banner?: ReactNode;
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
          <SoftLimitBanner workspace={workspace} role={role} />
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
