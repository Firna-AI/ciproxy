// A link inside a sentence, such as Terms in a notice. It takes the font and
// the size of the text around it and stays underlined, so it reads as a link
// among the words. It shows the Firna focus ring, as every other link of the
// app does. A link out of a page with work in progress, such as an open
// dialog or a review, opens a new tab, so the page keeps its place.

import { useFocusRing } from "@firna/ui/focusRing";

import { colors } from "@ciproxy/tokens";

export function InlineLink({
  label,
  href,
  newTab = false,
}: {
  label: string;
  href: string;
  /** Open the link in a new tab, so this page keeps its place. */
  newTab?: boolean;
}) {
  const focus = useFocusRing();
  return (
    <a
      href={href}
      target={newTab ? "_blank" : undefined}
      rel={newTab ? "noopener noreferrer" : undefined}
      {...focus.focusRingDomProps}
      onFocus={focus.onFocus}
      onBlur={focus.onBlur}
      style={{
        color: colors.ink,
        textDecoration: "underline",
        ...focus.focusRingVariables,
      }}
    >
      {label}
    </a>
  );
}
