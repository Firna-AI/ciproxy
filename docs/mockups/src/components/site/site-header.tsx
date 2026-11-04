// The top bar of the public site, shared by the marketing pages and the docs.
// Marketing: the wordmark, Pricing and Docs, then the Sign in link and the
// Start free button. The docs: the wordmark with "Docs", the search box, and
// the same actions. Every button that opens sign-in says Start free; the link
// serves people who already use ciproxy. On mobile, a menu button holds the
// links that do not fit.

import { Command, Github, Menu, Search } from "lucide-react";
import { MockLink, type Viewport } from "@mokly/mokly";
import { Button } from "@firna/ui/button";

import { LinkAction, noop } from "../actions.tsx";
import { Brand } from "../brand.tsx";
import { paths } from "../paths.ts";
import "./site-header.css";

/** The part of the site that a page belongs to. */
export type SiteSection = "home" | "pricing" | "docs";

type HeaderProps = { viewport: Viewport; section: SiteSection };

export function SiteHeader({ viewport, section }: HeaderProps) {
  const docs = section === "docs";
  return (
    <header
      className={`site-header${docs ? " site-header-docs" : ""} site-header-${viewport}`}
    >
      <div className="site-header-inner">
        <SiteTitle docs={docs} />
        {viewport === "desktop" ? (
          <DesktopParts section={section} />
        ) : (
          <MobileParts docs={docs} />
        )}
      </div>
    </header>
  );
}

function SiteTitle({ docs }: { docs: boolean }) {
  return (
    <div className="site-title">
      <MockLink to={paths.landing} className="site-home">
        <Brand />
      </MockLink>
      {docs ? (
        <>
          <span className="site-slash" aria-hidden="true">
            /
          </span>
          <MockLink to={paths.docsHome} className="site-section">
            Docs
          </MockLink>
        </>
      ) : null}
    </div>
  );
}

function navClass(active: boolean): string {
  return `site-nav-link${active ? " is-active" : ""}`;
}

function DesktopParts({ section }: { section: SiteSection }) {
  const docs = section === "docs";
  return (
    <>
      {docs ? (
        <MockLink to={paths.docsSearch} className="site-search">
          <Search size={15} strokeWidth={1.75} />
          <span className="site-search-text">Search docs</span>
          <kbd className="site-kbd" aria-label="Command K">
            <Command size={11} strokeWidth={2} />K
          </kbd>
        </MockLink>
      ) : (
        <nav className="site-nav" aria-label="Main">
          <MockLink
            to={paths.pricing}
            className={navClass(section === "pricing")}
          >
            Pricing
          </MockLink>
          <MockLink to={paths.docsHome} className={navClass(false)}>
            Docs
          </MockLink>
        </nav>
      )}
      <div className="site-actions">
        {docs ? (
          <>
            <MockLink to={paths.pricing} className={navClass(false)}>
              Pricing
            </MockLink>
            <span
              className="site-icon-link"
              title="The ciproxy agent on GitHub"
            >
              <Github size={18} strokeWidth={1.75} aria-label="GitHub" />
            </span>
          </>
        ) : null}
        <MockLink to={paths.signIn} className={navClass(false)}>
          Sign in
        </MockLink>
        <LinkAction to={paths.signIn} size="sm" tone="primary">
          Start free
        </LinkAction>
        {docs ? (
          <span className="site-menu">
            <MenuButton />
          </span>
        ) : null}
      </div>
    </>
  );
}

/** The menu of a narrow page: the docs sidebar, or the links of the site. */
function MenuButton() {
  return (
    <Button
      tone="plain"
      size="sm"
      shape="square"
      icon={Menu}
      accessibilityLabel="Menu"
      onPress={noop}
    />
  );
}

function MobileParts({ docs }: { docs: boolean }) {
  return (
    <div className="site-actions">
      {docs ? (
        <MockLink asChild to={paths.docsSearch}>
          <Button
            tone="plain"
            size="sm"
            shape="square"
            icon={Search}
            accessibilityLabel="Search docs"
            onPress={noop}
          />
        </MockLink>
      ) : (
        <LinkAction to={paths.signIn} size="sm" tone="primary">
          Start free
        </LinkAction>
      )}
      <MenuButton />
    </div>
  );
}
