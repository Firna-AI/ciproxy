// The small parts of the desktop frame: the setup item of the sidebar, and the
// top bar with the organization switch and the page as breadcrumbs.

import { ChevronsUpDown, Rocket } from "lucide-react";
import { MockLink } from "@mokly/mokly";
import { Avatar } from "@firna/ui/avatar";

import { org, user, type TierKey } from "../../fixtures/org.ts";
import { tiers } from "../../fixtures/tiers.ts";
import { paths } from "../paths.ts";
import { Tag } from "../status.tsx";
import type { SetupProgress } from "./app-shell.tsx";

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

export function TopBar({ tier, page }: { tier: TierKey; page: string }) {
  return (
    <header className="topbar">
      <nav className="topbar-crumbs" aria-label="Breadcrumb">
        <span className="org-switch">
          <Avatar
            label={org.initial}
            accessibilityLabel={org.login}
            size={22}
            shape="square"
          />
          <span className="org-name">{org.login}</span>
          <Tag>{tiers[tier].name}</Tag>
          <ChevronsUpDown size={14} className="org-chevron" />
        </span>
        <span className="topbar-slash" aria-hidden="true">
          /
        </span>
        <span className="topbar-page" aria-current="page">
          {page}
        </span>
      </nav>
      <Avatar
        label={user.initials}
        accessibilityLabel={user.name}
        size={28}
        tone="soft"
      />
    </header>
  );
}
