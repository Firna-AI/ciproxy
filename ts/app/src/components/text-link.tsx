// A link inside text, a list row, or a breadcrumb. A link to a page of the
// app is a real anchor that the router opens in place. A link out of the app
// is an anchor that opens a new tab. Both show the Firna focus ring. The
// plain look underlines on hover. The underlined look is the text link of the
// mockups, such as a step name that opens its log on GitHub: its underline
// stays, and a link out of the app shows the external icon after its label.

import { useState, type CSSProperties, type MouseEvent } from "react";
import { Link } from "expo-router";
import { useFocusRing } from "@firna/ui/focusRing";

import { colors, fonts } from "@ciproxy/tokens";

import { ExternalIcon } from "./icons.tsx";

export type TextLinkTone = "ink" | "muted";

/** The type size of the link: body text or small text. */
export type TextLinkSize = "body" | "small";

const sizes = {
  body: { fontSize: 14, lineHeight: "20px" },
  small: { fontSize: 13, lineHeight: "18px" },
} as const;

/** The plain look takes a tone and a weight; the underlined look has its own. */
type TextLinkLook =
  | {
      underlined?: false;
      tone?: TextLinkTone;
      /** Medium weight, such as a job name that opens its page. */
      strong?: boolean;
    }
  | {
      /**
       * The text link look of the mockups: ink2 at medium weight, with an
       * underline in the border colour that takes the text colour on hover.
       */
      underlined: true;
      tone?: undefined;
      strong?: undefined;
    };

type AnchorProps = {
  label: string;
  href: string;
  tone: TextLinkTone;
  size: TextLinkSize;
  strong: boolean;
  underlined: boolean;
  newTab: boolean;
  title: string | undefined;
  accessibilityLabel: string | undefined;
  /** Set by the router link. */
  onClick?: (event: MouseEvent<HTMLAnchorElement>) => void;
};

/** The colour, the weight, and the underline of the link. */
function lookStyle(props: AnchorProps, hovered: boolean): CSSProperties {
  if (props.underlined)
    return {
      display: "inline-flex",
      alignItems: "center",
      gap: 4,
      color: hovered ? colors.ink : colors.ink2,
      fontWeight: 500,
      textDecorationLine: "underline",
      textDecorationColor: hovered ? "currentColor" : colors.border2,
      textUnderlineOffset: "3px",
    };
  return {
    color: props.tone === "muted" && !hovered ? colors.muted : colors.ink,
    fontWeight: props.strong ? 500 : 400,
    textDecoration: hovered ? "underline" : "none",
  };
}

function Anchor(props: AnchorProps) {
  const { label, href, size, newTab, onClick } = props;
  const focus = useFocusRing();
  const [hovered, setHovered] = useState(false);
  const style: CSSProperties = {
    fontFamily: fonts.sans,
    ...sizes[size],
    overflowWrap: "anywhere",
    cursor: "pointer",
    ...lookStyle(props, hovered),
  };
  return (
    <a
      href={href}
      target={newTab ? "_blank" : undefined}
      rel={newTab ? "noopener noreferrer" : undefined}
      title={props.title}
      aria-label={props.accessibilityLabel}
      onClick={onClick}
      {...focus.focusRingDomProps}
      onFocus={focus.onFocus}
      onBlur={focus.onBlur}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{ ...style, ...focus.focusRingVariables }}
    >
      {label}
      {props.underlined && newTab ? (
        <ExternalIcon size={12} color={colors.muted} />
      ) : null}
    </a>
  );
}

export function TextLink({
  label,
  href,
  tone = "ink",
  size = "body",
  strong = false,
  underlined = false,
  external = false,
  title,
  accessibilityLabel,
}: {
  label: string;
  href: string;
  size?: TextLinkSize;
  /** A link out of the app, which opens a new tab. */
  external?: boolean;
  /** The tooltip, such as the place that the link opens. */
  title?: string;
  /** The words for a screen reader, when the label alone says too little. */
  accessibilityLabel?: string;
} & TextLinkLook) {
  const anchor = (
    <Anchor
      label={label}
      href={href}
      tone={tone}
      size={size}
      strong={strong}
      underlined={underlined}
      newTab={external}
      title={title}
      accessibilityLabel={accessibilityLabel}
    />
  );
  if (external) return anchor;
  return (
    <Link href={href} asChild>
      {anchor}
    </Link>
  );
}
