// The signed-in frame inside a design direction. The direction picks its
// chrome: a white sidebar with a breadcrumb top bar (topbar), a dark console
// sidebar (console), or a navy sidebar beside a white page header band
// (navy). On mobile, every direction has a top bar and a bottom tab bar.

import { BookOpen, ChevronDown, ChevronRight, Rocket } from "lucide-react";
import { MockLink } from "@mokly/mokly";
import { Avatar } from "@firna/ui/avatar";

import { org, user } from "../../fixtures/org.ts";
import { useRoutes, type Direction } from "../../theme/direction.ts";
import { Brand, BrandMark } from "../brand.tsx";
import { paths } from "../paths.ts";
import type { AppShellProps } from "./app-shell.tsx";
import { OrgSwitch, SetupItem, TopBar } from "./direction-shell-parts.tsx";
import { navItems } from "./nav.ts";
import "./direction-shell.css";
import "./direction-shell-chrome.css";
import "./direction-shell-mobile.css";

type ShellProps = AppShellProps & { direction: Direction };

export function DirectionShell(props: ShellProps) {
  return props.viewport === "mobile" ? (
    <MobileFrame {...props} />
  ) : (
    <DesktopFrame {...props} />
  );
}

function DesktopFrame({
  direction,
  tier,
  active,
  setup,
  banner,
  children,
}: ShellProps) {
  const route = useRoutes();
  const topbar = direction.shell === "topbar";
  const page = navItems.find((item) => item.key === active)?.label ?? "Set up";
  return (
    <div className={`ds ds-${direction.shell}`}>
      <aside className="ds-side">
        <div className="ds-side-inner">
          <div className="ds-brand">
            <Brand />
          </div>
          {topbar ? null : <OrgSwitch tier={tier} />}
          <nav className="ds-nav" aria-label="Main">
            {setup ? <SetupItem setup={setup} /> : null}
            {navItems.map((item) => (
              <MockLink
                key={item.key}
                to={route(item.to)}
                className={`ds-item${item.key === active ? " is-active" : ""}`}
                aria-current={item.key === active ? "page" : undefined}
              >
                <item.icon size={16} strokeWidth={1.75} />
                <span className="ds-label">{item.label}</span>
              </MockLink>
            ))}
          </nav>
          <div className="ds-foot">
            <span className="ds-item">
              <BookOpen size={16} strokeWidth={1.75} />
              <span className="ds-label">Docs</span>
            </span>
            {topbar ? null : (
              <div className="ds-user">
                <Avatar
                  label={user.initials}
                  accessibilityLabel={user.name}
                  size={24}
                  tone="soft"
                />
                <span className="ds-label">{user.name}</span>
              </div>
            )}
          </div>
        </div>
      </aside>
      <div className="ds-body">
        {topbar ? <TopBar tier={tier} page={page} /> : null}
        <main className="ds-main">
          {banner}
          {children}
        </main>
      </div>
    </div>
  );
}

function MobileFrame({ active, setup, banner, children }: ShellProps) {
  const route = useRoutes();
  return (
    <div className="dm">
      <header className="dm-top">
        <span className="dm-org">
          <BrandMark size={24} />
          <span className="dm-divider" aria-hidden="true" />
          <span className="dm-org-name">{org.login}</span>
          <ChevronDown size={14} className="dm-chevron" />
        </span>
        <Avatar
          label={user.initials}
          accessibilityLabel={user.name}
          size={26}
          tone="soft"
        />
      </header>
      {setup && !setup.active ? (
        <MockLink to={paths.onboarding} className="dm-setup">
          <Rocket size={15} strokeWidth={1.75} />
          <span className="ds-label">Finish setup</span>
          <span className="ds-count">
            {setup.done} of {setup.total}
          </span>
          <ChevronRight size={15} />
        </MockLink>
      ) : null}
      <main className="dm-main">
        {banner}
        {children}
      </main>
      <nav className="dm-tabs" aria-label="Main">
        {navItems.map((item) => (
          <MockLink
            key={item.key}
            to={route(item.to)}
            className={`dm-tab${item.key === active ? " is-active" : ""}`}
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
