// Buttons and link buttons. A button acts in place, through the Firna
// Button. A link button is a real anchor, so it opens in a new tab, shows its
// address, and works before the page script runs. It takes its look from the
// Firna button styles and its focus glow from the Firna focus ring.

import { useMemo, useState, type CSSProperties, type MouseEvent } from "react";
import type { IconComponent } from "@firna/ui";
import { Button, buttonIconSize, createButtonStyles } from "@firna/ui/button";
import { useFocusRing } from "@firna/ui/focusRing";
import { useSharedUiTheme } from "@firna/ui/theme";

export type Tone = "primary" | "secondary" | "ghost";
export type Size = "sm" | "md" | "lg";

type Common = {
  label: string;
  tone?: Tone;
  size?: Size;
  icon?: IconComponent;
  block?: boolean;
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
  return (
    <Button
      tone={tone}
      size={size}
      icon={icon}
      block={block}
      busy={busy}
      disabled={disabled}
      onPress={disabled ? undefined : onPress}
    >
      {label}
    </Button>
  );
}

function linkStyle(
  styles: ReturnType<typeof createButtonStyles>,
  tone: Tone,
  hovered: boolean,
  block: boolean,
): CSSProperties {
  const { button, label } = styles;
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
    height: button.height,
    padding: `0 ${button.paddingHorizontal}px`,
    width: block ? "100%" : undefined,
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
    whiteSpace: "nowrap",
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
}: Common & {
  href: string;
  newTab?: boolean;
  /** Show only the icon; the label stays the accessible name. */
  hideLabel?: boolean;
  /** Called before the browser follows a plain click. */
  onClick?: (event: MouseEvent<HTMLAnchorElement>) => void;
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
      onClick={onClick}
      aria-label={hideLabel ? label : undefined}
      title={hideLabel ? label : undefined}
      style={{
        ...linkStyle(styles, tone, hovered, block),
        ...focus.focusRingVariables,
        color,
      }}
    >
      {Icon ? <Icon size={buttonIconSize(size)} color={color} /> : null}
      {hideLabel ? null : <span>{label}</span>}
    </a>
  );
}
