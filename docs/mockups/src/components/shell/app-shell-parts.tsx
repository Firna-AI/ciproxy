// The small parts of the desktop frame: the setup item of the sidebar, and the
// top bar with the workspace switch, the page as breadcrumbs, and the avatar
// of the signed-in person. The switch changes the workspace, which holds one
// or more GitHub organizations.

import type { ReactNode } from "react";
import { ChevronsUpDown, Rocket } from "lucide-react";
import { MockLink } from "@mokly/mokly";
import { Avatar } from "@firna/ui/avatar";

import type { Role } from "../../fixtures/members.ts";
import { workspace, type TierKey } from "../../fixtures/org.ts";
import { tiers } from "../../fixtures/tiers.ts";
import { paths } from "../paths.ts";
import { Tag } from "../status.tsx";
import { AccountMenu } from "./account-menu.tsx";
import { WorkspaceMenu } from "./workspace-menu.tsx";
import type { SetupProgress, ShellPerson } from "./app-shell.tsx";

export function SetupItem({ setup }: { setup: SetupProgress }) {
  return (
    <MockLink
      to={paths.onboarding}
      className={`side-item${setup.active ? " is-active" : ""}`}
    >
      <Rocket size={16} strokeWidth={1.75} />
      <span className="side-label">Set up</span>
      <span className="side-count">
        {setup.done} of {setup.total}
      </span>
    </MockLink>
  );
}

export function TopBar({
  tier,
  page,
  account,
  workspaceMenu,
}: {
  tier: TierKey;
  page: string;
  /** The avatar of the signed-in person, with its account menu. */
  account: ReactNode;
  /** The menu of the workspace switch is open. */
  workspaceMenu: boolean;
}) {
  return (
    <header className="topbar">
      <nav className="topbar-crumbs" aria-label="Breadcrumb">
        <span className="org-switch workspace-switch" title="Switch workspace">
          <Avatar
            label={workspace.initial}
            accessibilityLabel={workspace.name}
            size={22}
            shape="square"
          />
          <span className="org-name">{workspace.name}</span>
          <Tag>{tiers[tier].name}</Tag>
          <ChevronsUpDown size={14} className="org-chevron" />
          {workspaceMenu ? <WorkspaceMenu /> : null}
        </span>
        <span className="topbar-slash" aria-hidden="true">
          /
        </span>
        <span className="topbar-page" aria-current="page">
          {page}
        </span>
      </nav>
      {account}
    </header>
  );
}

/** The avatar of the top bar, with the account menu when it is open. */
export function Account({
  person,
  role,
  open,
  size,
}: {
  person: ShellPerson;
  role: Role;
  open: boolean;
  size: number;
}) {
  return (
    <span className="account">
      <Avatar
        label={person.initials}
        accessibilityLabel={person.name}
        size={size}
        tone="soft"
      />
      {open ? <AccountMenu person={person} role={role} /> : null}
    </span>
  );
}
