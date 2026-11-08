// Buttons and link buttons. A button acts in place, through the Firna
// Button. A link button is a real anchor, so it opens in a new tab, shows its
// address, and works before the page script runs. It takes its look from the
// Firna button styles and its focus glow from the Firna focus ring.
//
// Both grow with their label, because a label can hold a name of any length.
// A label that fits on one line keeps the Firna box. A longer label wraps
// between words, and inside a word that is wider than the control. A control
// is never wider than its row, and a short one keeps its width in a row.

import { useMemo, useState, type CSSProperties, type MouseEvent } from "react";
import type { IconComponent } from "@firna/ui";
import { Button, buttonIconSize, createButtonStyles } from "@firna/ui/button";
import { useFocusRing } from "@firna/ui/focusRing";
import { useSharedUiTheme } from "@firna/ui/theme";

export type Tone = "primary" | "secondary" | "ghost";
export type Size = "sm" | "md" | "lg";

type ButtonStyles = ReturnType<typeof createButtonStyles>;

type Common = {
  label: string;
  tone?: Tone;
  size?: Size;
  icon?: IconComponent;
  block?: boolean;
};

/**
 * The box of a control that grows with its label. The Firna track height is
 * its least height, and the vertical padding centres one label line in that
 * track, so each more line adds one line height.
 */
function growingBox({ button, label }: ButtonStyles) {
  return {
    height: "auto",
    minHeight: button.height,
    paddingVertical:
      (button.height - 2 * button.borderWidth - label.lineHeight) / 2,
    flexShrink: 0,
    maxWidth: "100%",
  } as const;
}

/**
 * The label of a control that grows. Its lines are centred, and it can be
 * narrower than its longest word. That word then breaks at the edge of the
 * control: React Native Web text breaks it, and a link label sets the same
 * rule.
 */
const wrappingLabel = { minWidth: 0, textAlign: "center" } as const;

const wrappingLinkLabel: CSSProperties = {
  ...wrappingLabel,
  overflowWrap: "break-word",
};

export function ActionButton({
  label,
  tone = "secondary",
  size = "md",
  icon,
  block,
  busy,
  disabled,
  onPress,
}: Common & { busy?: boolean; disabled?: boolean; onPress: () => void }) {
  const theme = useSharedUiTheme();
  const box = useMemo(
    () => growingBox(createButtonStyles(theme, size)),
    [theme, size],
  );
  return (
    <Button
      tone={tone}
      size={size}
      icon={icon}
      block={block}
      busy={busy}
      disabled={disabled}
      onPress={disabled ? undefined : onPress}
      style={box}
      labelStyle={wrappingLabel}
    >
      {label}
    </Button>
  );
}

function linkStyle(
  styles: ButtonStyles,
  tone: Tone,
  hovered: boolean,
  block: boolean,
): CSSProperties {
  const { button, label } = styles;
  const box = growingBox(styles);
  const fill =
    tone === "primary"
      ? styles.primary
      : tone === "ghost"
        ? hovered
          ? { ...styles.ghost, ...styles.ghostHover }
          : styles.ghost
        : hovered
          ? { ...button, ...styles.secondaryHover }
          : button;
  return {
    display: "inline-flex",
    alignItems: "center",
    justifyContent: "center",
    gap: button.gap,
    boxSizing: "border-box",
    height: box.height,
    minHeight: box.minHeight,
    padding: `${box.paddingVertical}px ${button.paddingHorizontal}px`,
    flexShrink: box.flexShrink,
    width: block ? "100%" : undefined,
    maxWidth: box.maxWidth,
    borderRadius: button.borderRadius,
    borderStyle: "solid",
    borderWidth: button.borderWidth,
    borderColor: "borderColor" in fill ? fill.borderColor : button.borderColor,
    backgroundColor: fill.backgroundColor,
    fontFamily: label.fontFamily,
    fontSize: label.fontSize,
    fontWeight: label.fontWeight,
    lineHeight: `${label.lineHeight}px`,
    textDecoration: "none",
    whiteSpace: "normal",
    cursor: "pointer",
  };
}

/** A link that looks like a button. */
export function ActionLink({
  label,
  href,
  tone = "secondary",
  size = "md",
  icon: Icon,
  block = false,
  newTab = false,
  hideLabel = false,
  onClick,
  onActivate,
}: Common & {
  href: string;
  newTab?: boolean;
  /** Show only the icon; the label stays the accessible name. */
  hideLabel?: boolean;
  /** The raw click hook, also used by router links. */
  onClick?: (event: MouseEvent<HTMLAnchorElement>) => void;
  /** Run an action for a plain current-tab activation instead of following the link. */
  onActivate?: () => void;
}) {
  const theme = useSharedUiTheme();
  const styles = useMemo(() => createButtonStyles(theme, size), [theme, size]);
  const focus = useFocusRing();
  const [hovered, setHovered] = useState(false);
  const color =
    tone === "primary"
      ? theme.colors.onSolid
      : tone === "ghost"
        ? theme.colors.primaryDeep
        : theme.colors.ink;
  return (
    <a
      href={href}
      target={newTab ? "_blank" : undefined}
      rel={newTab ? "noopener noreferrer" : undefined}
      {...focus.focusRingDomProps}
      onFocus={focus.onFocus}
      onBlur={focus.onBlur}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      data-ciproxy-action={onActivate ? "true" : undefined}
      onClick={
        onActivate
          ? (event) => {
              if (
                event.defaultPrevented ||
                event.button !== 0 ||
                newTab ||
                event.metaKey ||
                event.ctrlKey ||
                event.shiftKey ||
                event.altKey
              )
                return;
              event.preventDefault();
              onActivate();
            }
          : onClick
      }
      aria-label={hideLabel ? label : undefined}
      title={hideLabel ? label : undefined}
      style={{
        ...linkStyle(styles, tone, hovered, block),
        ...focus.focusRingVariables,
        color,
      }}
    >
      {Icon ? <Icon size={buttonIconSize(size)} color={color} /> : null}
      {hideLabel ? null : <span style={wrappingLinkLabel}>{label}</span>}
    </a>
  );
}
