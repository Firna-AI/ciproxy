// The small parts of the direction frame: the organization switch, the setup
// item of the sidebar, and the breadcrumb top bar of the topbar chrome.

import { ChevronsUpDown, Rocket } from "lucide-react";
import { MockLink } from "@mokly/mokly";
import { Avatar } from "@firna/ui/avatar";

import { org, user, type TierKey } from "../../fixtures/org.ts";
import { tiers } from "../../fixtures/tiers.ts";
import { paths } from "../paths.ts";
import type { SetupProgress } from "./app-shell.tsx";

export function OrgSwitch({ tier }: { tier: TierKey }) {
  return (
    <div className="ds-org">
      <Avatar
        label={org.initial}
        accessibilityLabel={org.login}
        size={22}
        shape="square"
      />
      <span className="ds-org-text">
        <span className="ds-org-name">{org.login}</span>
        <span className="ds-org-tier">{tiers[tier].name}</span>
      </span>
      <ChevronsUpDown size={14} className="ds-org-chevron" />
    </div>
  );
}

export function SetupItem({ setup }: { setup: SetupProgress }) {
  return (
    <MockLink
      to={paths.onboarding}
      className={`ds-item${setup.active ? " is-active" : ""}`}
    >
      <Rocket size={16} strokeWidth={1.75} />
      <span className="ds-label">Set up</span>
      <span className="ds-count">
        {setup.done} of {setup.total}
      </span>
    </MockLink>
  );
}

export function TopBar({ tier, page }: { tier: TierKey; page: string }) {
  return (
    <header className="ds-top">
      <nav className="ds-crumbs" aria-label="Breadcrumb">
        <OrgSwitch tier={tier} />
        <span className="ds-slash" aria-hidden="true">
          /
        </span>
        <span className="ds-crumb" aria-current="page">
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
