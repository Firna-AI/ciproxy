// One item of the main navigation: a row of the sidebar, an icon of the
// rail, or a tab of the phone bar. It is a real link that the router opens in
// place, and it marks the current page.

import { useState, type CSSProperties, type MouseEvent } from "react";
import { Link } from "expo-router";
import { useFocusRing } from "@firna/ui/focusRing";

import { colors, fonts, navColors, radii } from "@ciproxy/tokens";

import type { WorkspacePage } from "../../routing/pages.ts";
import { siteLinks } from "../../view/page-model.ts";
import type { NavItem } from "../../view/shell.ts";
import {
  DocsIcon,
  JobsIcon,
  MachinesIcon,
  PlanIcon,
  SavingsIcon,
  SettingsIcon,
  SetupIcon,
} from "../icons.tsx";

export const navIcons = {
  setup: SetupIcon,
  jobs: JobsIcon,
  machines: MachinesIcon,
  savings: SavingsIcon,
  billing: PlanIcon,
  settings: SettingsIcon,
} as const satisfies Record<WorkspacePage, unknown>;

export type NavVariant = "side" | "rail" | "tab";

type AnchorProps = {
  item: NavItem;
  active: boolean;
  variant: NavVariant;
  /** Set by the router link. */
  href?: string;
  onClick?: (event: MouseEvent<HTMLAnchorElement>) => void;
};

function itemStyle(
  variant: NavVariant,
  active: boolean,
  hovered: boolean,
): CSSProperties {
  const base: CSSProperties = {
    display: "flex",
    alignItems: "center",
    boxSizing: "border-box",
    fontFamily: fonts.sans,
    fontWeight: 500,
    textDecoration: "none",
    color: active || hovered ? colors.ink : navColors.label,
  };
  if (variant === "tab") {
    return {
      ...base,
      flexDirection: "column",
      gap: 3,
      padding: "4px 0",
      fontSize: 10.5,
      lineHeight: "14px",
      color: active ? colors.ink : colors.muted,
    };
  }
  return {
    ...base,
    gap: 10,
    height: 32,
    padding: variant === "rail" ? 0 : "0 10px",
    width: variant === "rail" ? 40 : undefined,
    justifyContent: variant === "rail" ? "center" : undefined,
    borderRadius: radii.md,
    fontSize: 14,
    backgroundColor: active
      ? colors.primarySoft
      : hovered
        ? colors.soft
        : "transparent",
  };
}

function NavAnchor({ item, active, variant, href, onClick }: AnchorProps) {
  const focus = useFocusRing();
  const [hovered, setHovered] = useState(false);
  const Icon = navIcons[item.key];
  const iconColor = active || hovered ? colors.ink : navColors.icon;
  return (
    <a
      href={href ?? item.href}
      onClick={onClick}
      aria-current={active ? "page" : undefined}
      aria-label={variant === "rail" ? item.label : undefined}
      title={variant === "rail" ? item.label : undefined}
      {...focus.focusRingDomProps}
      onFocus={focus.onFocus}
      onBlur={focus.onBlur}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        ...itemStyle(variant, active, hovered),
        ...focus.focusRingVariables,
      }}
    >
      <Icon
        size={variant === "tab" ? 19 : 16}
        strokeWidth={variant === "tab" ? 1.6 : 1.75}
        color={variant === "tab" ? undefined : iconColor}
      />
      {variant === "rail" ? null : (
        <span
          style={{
            flex: variant === "side" ? 1 : undefined,
            whiteSpace: "nowrap",
          }}
        >
          {variant === "tab" ? item.mobileLabel : item.label}
        </span>
      )}
      {variant === "side" && item.count ? (
        <span
          style={{
            color: navColors.label,
            fontFamily: fonts.mono,
            fontSize: 11,
          }}
        >
          {item.count}
        </span>
      ) : null}
    </a>
  );
}

/** The docs link at the foot of the sidebar, in the look of a nav row. */
export function DocsLink({ rail }: { rail: boolean }) {
  const focus = useFocusRing();
  const [hovered, setHovered] = useState(false);
  const variant: NavVariant = rail ? "rail" : "side";
  return (
    <a
      href={siteLinks.docs}
      aria-label={rail ? "Docs" : undefined}
      title={rail ? "Docs" : undefined}
      {...focus.focusRingDomProps}
      onFocus={focus.onFocus}
      onBlur={focus.onBlur}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        ...itemStyle(variant, false, hovered),
        ...focus.focusRingVariables,
      }}
    >
      <DocsIcon size={16} color={hovered ? colors.ink : navColors.icon} />
      {rail ? null : <span style={{ flex: 1 }}>Docs</span>}
    </a>
  );
}

export function NavLink(props: Omit<AnchorProps, "href" | "onClick">) {
  return (
    <Link href={props.item.href} asChild>
      <NavAnchor {...props} />
    </Link>
  );
}
