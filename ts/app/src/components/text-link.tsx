// A link inside text, a list row, or a breadcrumb. A link to a page of the
// app is a real anchor that the router opens in place. A link out of the app
// is an anchor that opens a new tab. Both underline on hover and show the
// Firna focus ring.

import { useState, type CSSProperties, type MouseEvent } from "react";
import { Link } from "expo-router";
import { useFocusRing } from "@firna/ui/focusRing";

import { colors, fonts } from "@ciproxy/tokens";

export type TextLinkTone = "ink" | "muted";

/** The type size of the link: body text or small text. */
export type TextLinkSize = "body" | "small";

const sizes = {
  body: { fontSize: 14, lineHeight: "20px" },
  small: { fontSize: 13, lineHeight: "18px" },
} as const;

type AnchorProps = {
  label: string;
  href: string;
  tone: TextLinkTone;
  size: TextLinkSize;
  strong: boolean;
  newTab: boolean;
  /** Set by the router link. */
  onClick?: (event: MouseEvent<HTMLAnchorElement>) => void;
};

function Anchor(props: AnchorProps) {
  const { label, href, tone, size, strong, newTab, onClick } = props;
  const focus = useFocusRing();
  const [hovered, setHovered] = useState(false);
  const style: CSSProperties = {
    color: tone === "muted" && !hovered ? colors.muted : colors.ink,
    fontFamily: fonts.sans,
    ...sizes[size],
    fontWeight: strong ? 500 : 400,
    textDecoration: hovered ? "underline" : "none",
    overflowWrap: "anywhere",
    cursor: "pointer",
  };
  return (
    <a
      href={href}
      target={newTab ? "_blank" : undefined}
      rel={newTab ? "noopener noreferrer" : undefined}
      onClick={onClick}
      {...focus.focusRingDomProps}
      onFocus={focus.onFocus}
      onBlur={focus.onBlur}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{ ...style, ...focus.focusRingVariables }}
    >
      {label}
    </a>
  );
}

export function TextLink({
  label,
  href,
  tone = "ink",
  size = "body",
  strong = false,
  external = false,
}: {
  label: string;
  href: string;
  tone?: TextLinkTone;
  size?: TextLinkSize;
  /** Medium weight, such as a job name that opens its page. */
  strong?: boolean;
  /** A link out of the app, which opens a new tab. */
  external?: boolean;
}) {
  const anchor = (
    <Anchor
      label={label}
      href={href}
      tone={tone}
      size={size}
      strong={strong}
      newTab={external}
    />
  );
  if (external) return anchor;
  return (
    <Link href={href} asChild>
      {anchor}
    </Link>
  );
}
