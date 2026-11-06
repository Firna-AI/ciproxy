// The signed-in frame of every workspace page. It picks the desktop, rail,
// or phone frame by the window, and puts the banner of the organization soft
// limit and a failed menu action above the page.

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
import { text } from "../text.ts";
import { useViewport } from "../viewport.ts";
import { DesktopShell } from "./desktop-shell.tsx";
import { MobileShell } from "./mobile-shell.tsx";
import { useShellActions } from "./shell-actions.ts";

function Banner({
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
  children,
}: {
  session: SessionView;
  workspace: WorkspaceView;
  role: WorkspaceRole;
  page: WorkspacePage;
  setup: SetupProgress | null;
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
        <Banner workspace={workspace} role={role} />
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
