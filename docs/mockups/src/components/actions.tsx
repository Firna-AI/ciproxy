// Buttons and links of the static mockups. A Firna Button without onPress
// renders disabled, so every enabled button gets a no-op handler. A button
// that leads to another screen is a Mokly link that keeps the button's look.

import type { ReactNode } from "react";
import { MockLink } from "@mokly/mokly";
import type { IconComponent } from "@firna/ui";
import { Button, type ButtonTone } from "@firna/ui/button";

import type { CataloguePath } from "./paths.ts";

/** The handler of a static button: the mockup records no action. */
export const noop = (): void => undefined;

type ActionProps = {
  children: ReactNode;
  tone?: ButtonTone;
  size?: "sm" | "md" | "lg";
  icon?: IconComponent;
  block?: boolean;
  disabled?: boolean;
  busy?: boolean;
};

/** A button that acts in place, such as Copy or Save. */
export function Action({
  children,
  tone = "secondary",
  ...props
}: ActionProps) {
  return (
    <Button tone={tone} onPress={props.disabled ? undefined : noop} {...props}>
      {children}
    </Button>
  );
}

/** A button that opens another screen of the catalogue. */
export function LinkAction({
  to,
  ...props
}: ActionProps & { to: CataloguePath }) {
  return (
    <MockLink asChild to={to}>
      <Action {...props} />
    </MockLink>
  );
}

/** An inline text link to another screen of the catalogue. */
export function TextLink({
  to,
  children,
}: {
  to: CataloguePath;
  children: ReactNode;
}) {
  return (
    <MockLink to={to} className="text-link">
      {children}
    </MockLink>
  );
}
