// The signed-in frame of every dashboard screen: a sidebar on desktop, and a
// top bar with a bottom tab bar on mobile. A banner, such as a soft-limit
// warning, sits above the page content on every page of the organization.
// Inside a design direction, the direction's frame renders instead.

import type { ReactNode } from "react";
import {
  BookOpen,
  ChevronDown,
  ChevronRight,
  ChevronsUpDown,
  Rocket,
} from "lucide-react";
import { MockLink, type Viewport } from "@mokly/mokly";
import { Avatar } from "@firna/ui/avatar";

import { org, user, type TierKey } from "../../fixtures/org.ts";
import { tiers } from "../../fixtures/tiers.ts";
import { useDirection } from "../../theme/direction.ts";
import { Brand, BrandMark } from "../brand.tsx";
import { paths } from "../paths.ts";
import { DirectionShell } from "./direction-shell.tsx";
import { navItems, type NavKey } from "./nav.ts";
import "./app-shell.css";
import "./app-shell-rail.css";

export type SetupProgress = { done: number; total: number; active: boolean };

export type AppShellProps = {
  viewport: Viewport;
  tier: TierKey;
  active?: NavKey;
  /** While setup is open, the sidebar leads with a Set up item. */
  setup?: SetupProgress;
  banner?: ReactNode;
  children: ReactNode;
};

export function AppShell(props: AppShellProps) {
  const direction = useDirection();
  if (direction) return <DirectionShell {...props} direction={direction} />;
  return props.viewport === "mobile" ? (
    <MobileShell {...props} />
  ) : (
    <DesktopShell {...props} />
  );
}

function DesktopShell({
  tier,
  active,
  setup,
  banner,
  children,
}: AppShellProps) {
  return (
    <div className="shell-desktop">
      <aside className="sidebar">
        <div className="sidebar-inner">
          <div className="sidebar-brand">
            <Brand />
          </div>
          <div className="org-switch">
            <Avatar
              label={org.initial}
              accessibilityLabel={org.login}
              size={28}
              shape="square"
            />
            <span className="org-switch-text">
              <span className="org-switch-name">{org.login}</span>
              <span className="caption">{tiers[tier].name}</span>
            </span>
            <ChevronsUpDown size={15} className="icon-muted" />
          </div>
          <nav className="sidebar-nav" aria-label="Main">
            {setup ? (
              <MockLink
                to={paths.onboarding}
                className={`nav-item${setup.active ? " is-active" : ""}`}
              >
                <Rocket size={17} />
                <span className="nav-label">Set up</span>
                <span className="nav-badge">
                  {setup.done} of {setup.total}
                </span>
              </MockLink>
            ) : null}
            {navItems.map((item) => (
              <MockLink
                key={item.key}
                to={item.to}
                className={`nav-item${item.key === active ? " is-active" : ""}`}
                aria-current={item.key === active ? "page" : undefined}
              >
                <item.icon size={17} />
                <span className="nav-label">{item.label}</span>
              </MockLink>
            ))}
          </nav>
          <div className="sidebar-foot">
            <span className="nav-item">
              <BookOpen size={17} />
              <span className="nav-label">Docs</span>
            </span>
            <div className="user-row">
              <Avatar
                label={user.initials}
                accessibilityLabel={user.name}
                size={28}
              />
              <span className="small">{user.name}</span>
            </div>
          </div>
        </div>
      </aside>
      <main className="shell-main">
        {banner}
        {children}
      </main>
    </div>
  );
}

function MobileShell({ active, setup, banner, children }: AppShellProps) {
  return (
    <div className="shell-mobile">
      <header className="m-top">
        <span className="row">
          <BrandMark size={24} />
          <span className="m-org">
            {org.login}
            <ChevronDown size={15} className="icon-muted" />
          </span>
        </span>
        <Avatar
          label={user.initials}
          accessibilityLabel={user.name}
          size={28}
        />
      </header>
      {setup && !setup.active ? (
        <MockLink to={paths.onboarding} className="m-setup">
          <Rocket size={16} />
          <span className="nav-label">Finish setup</span>
          <span className="nav-badge">
            {setup.done} of {setup.total}
          </span>
          <ChevronRight size={16} />
        </MockLink>
      ) : null}
      <main className="m-main">
        {banner}
        {children}
      </main>
      <nav className="m-tabs" aria-label="Main">
        {navItems.map((item) => (
          <MockLink
            key={item.key}
            to={item.to}
            className={`m-tab${item.key === active ? " is-active" : ""}`}
            aria-current={item.key === active ? "page" : undefined}
          >
            <item.icon size={20} />
            <span>{item.mobileLabel}</span>
          </MockLink>
        ))}
      </nav>
    </div>
  );
}
