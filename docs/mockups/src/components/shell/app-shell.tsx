// The signed-in frame of every dashboard screen. On desktop: a white sidebar
// with the main navigation beside a top bar that holds the organization and
// the page as breadcrumbs. On mobile: a top bar and a bottom tab bar. A
// banner, such as a soft-limit warning, sits above the page content on every
// page of the organization.

import type { ReactNode } from "react";
import { BookOpen, ChevronDown, ChevronRight, Rocket } from "lucide-react";
import { MockLink, type Viewport } from "@mokly/mokly";
import { Avatar } from "@firna/ui/avatar";

import type { Role } from "../../fixtures/members.ts";
import { org, user, type TierKey } from "../../fixtures/org.ts";
import { Brand, BrandMark } from "../brand.tsx";
import { paths } from "../paths.ts";
import { SetupItem, TopBar } from "./app-shell-parts.tsx";
import { navFor, navItems, type NavKey } from "./nav.ts";
import "./app-shell.css";
import "./app-shell-mobile.css";
import "./app-shell-rail.css";

export type SetupProgress = { done: number; total: number; active: boolean };

/** The signed-in person: Cal Moore, unless a screen shows another person. */
export type ShellPerson = { name: string; initials: string };

export type AppShellProps = {
  viewport: Viewport;
  tier: TierKey;
  active?: NavKey;
  person?: ShellPerson;
  /** The role of the person, which decides the navigation. */
  role?: Role;
  /** While setup is open, the sidebar leads with a Set up item. */
  setup?: SetupProgress;
  banner?: ReactNode;
  children: ReactNode;
};

export function AppShell(props: AppShellProps) {
  return props.viewport === "mobile" ? (
    <MobileShell {...props} />
  ) : (
    <DesktopShell {...props} />
  );
}

function DesktopShell({
  tier,
  active,
  person = user,
  role = "admin",
  setup,
  banner,
  children,
}: AppShellProps) {
  const page = navItems.find((item) => item.key === active)?.label ?? "Set up";
  return (
    <div className="shell">
      <aside className="side">
        <div className="side-inner">
          <div className="side-brand">
            <Brand />
          </div>
          <nav className="side-nav" aria-label="Main">
            {setup ? <SetupItem setup={setup} /> : null}
            {navFor(role).map((item) => (
              <MockLink
                key={item.key}
                to={item.to}
                className={`side-item${item.key === active ? " is-active" : ""}`}
                aria-current={item.key === active ? "page" : undefined}
              >
                <item.icon size={16} strokeWidth={1.75} />
                <span className="side-label">{item.label}</span>
              </MockLink>
            ))}
          </nav>
          <div className="side-foot">
            <MockLink to={paths.docsHome} className="side-item">
              <BookOpen size={16} strokeWidth={1.75} />
              <span className="side-label">Docs</span>
            </MockLink>
          </div>
        </div>
      </aside>
      <div className="shell-body">
        <TopBar tier={tier} page={page} person={person} />
        <main className="shell-main">
          {banner}
          {children}
        </main>
      </div>
    </div>
  );
}

function MobileShell({
  active,
  person = user,
  role = "admin",
  setup,
  banner,
  children,
}: AppShellProps) {
  return (
    <div className="m-shell">
      <header className="m-top">
        <span className="m-org">
          <BrandMark size={18} />
          <span className="m-divider" aria-hidden="true" />
          <span className="m-org-name">{org.login}</span>
          <ChevronDown size={14} className="m-chevron" />
        </span>
        <Avatar
          label={person.initials}
          accessibilityLabel={person.name}
          size={26}
          tone="soft"
        />
      </header>
      {setup && !setup.active ? (
        <MockLink to={paths.onboarding} className="m-setup">
          <Rocket size={15} strokeWidth={1.75} />
          <span className="m-setup-label">Finish setup</span>
          <span className="m-count">
            {setup.done} of {setup.total}
          </span>
          <ChevronRight size={15} />
        </MockLink>
      ) : null}
      <main className="m-main">
        {banner}
        {children}
      </main>
      <nav className="m-tabs" aria-label="Main">
        {navFor(role).map((item) => (
          <MockLink
            key={item.key}
            to={item.to}
            className={`m-tab${item.key === active ? " is-active" : ""}`}
            aria-current={item.key === active ? "page" : undefined}
          >
            <item.icon size={19} strokeWidth={1.6} />
            <span>{item.mobileLabel}</span>
          </MockLink>
        ))}
      </nav>
    </div>
  );
}
