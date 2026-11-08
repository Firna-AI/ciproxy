// The icons of the dashboard: small stroke drawings on a 24-unit grid, drawn
// with the current text colour, as inline SVG like the brand mark. Each one
// fits the icon prop of the Firna controls.

import type { ReactNode } from "react";
import type { IconComponent, IconComponentProps } from "@firna/ui";

function icon(name: string, shapes: ReactNode): IconComponent {
  function Icon({ color, size = 16, strokeWidth = 1.75 }: IconComponentProps) {
    return (
      <svg
        aria-hidden="true"
        width={size}
        height={size}
        viewBox="0 0 24 24"
        fill="none"
        stroke={color ?? "currentColor"}
        strokeWidth={strokeWidth}
        strokeLinecap="round"
        strokeLinejoin="round"
        style={{ flexShrink: 0 }}
      >
        {shapes}
      </svg>
    );
  }
  Icon.displayName = `${name}Icon`;
  return Icon;
}

export const PlusIcon = icon("Plus", <path d="M12 5v14M5 12h14" />);

export const CheckIcon = icon("Check", <path d="M5 12.5l4.5 4.5L19 7.5" />);

export const ExternalIcon = icon(
  "External",
  <path d="M14 4h6v6M20 4l-9 9M18 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1h5" />,
);

export const SwitchIcon = icon(
  "Switch",
  <path d="M8 9l4-4 4 4M8 15l4 4 4-4" />,
);

export const ChevronIcon = icon("Chevron", <path d="M9 6l6 6-6 6" />);

export const InfoIcon = icon(
  "Info",
  <>
    <circle cx="12" cy="12" r="9" />
    <path d="M12 11v5M12 8h.01" />
  </>,
);

export const WarningIcon = icon(
  "Warning",
  <path d="M12 4l9 16H3zM12 10v4M12 17h.01" />,
);

export const AlertIcon = icon(
  "Alert",
  <>
    <circle cx="12" cy="12" r="9" />
    <path d="M12 8v5M12 16h.01" />
  </>,
);

export const DoneIcon = icon(
  "Done",
  <>
    <circle cx="12" cy="12" r="9" />
    <path d="M8.5 12.5l2.5 2.5 4.5-5" />
  </>,
);

export const JobsIcon = icon(
  "Jobs",
  <path d="M4 6.5l1.5 1.5 3-3M4 14.5l1.5 1.5 3-3M12 7h8M12 15h8M12 20h8" />,
);

export const MachinesIcon = icon(
  "Machines",
  <>
    <rect x="4" y="4" width="16" height="7" rx="1.5" />
    <rect x="4" y="13" width="16" height="7" rx="1.5" />
    <path d="M8 7.5h.01M8 16.5h.01" />
  </>,
);

export const SavingsIcon = icon(
  "Savings",
  <>
    <circle cx="12" cy="12" r="9" />
    <path d="M14.5 9.5c-.4-.9-1.4-1.5-2.5-1.5-1.4 0-2.5.8-2.5 2s1.1 1.6 2.5 2 2.5.8 2.5 2-1.1 2-2.5 2c-1.1 0-2.1-.6-2.5-1.5M12 6.5V8M12 16v1.5" />
  </>,
);

export const PlanIcon = icon(
  "Plan",
  <path d="M4.5 17a8.5 8.5 0 1 1 15 0M12 13l3.5-4.5M12 13h.01" />,
);

export const SettingsIcon = icon(
  "Settings",
  <>
    <path d="M4 7h10M18 7h2M4 17h4M12 17h8" />
    <circle cx="16" cy="7" r="2" />
    <circle cx="10" cy="17" r="2" />
  </>,
);

export const SetupIcon = icon("Setup", <path d="M5 21V4M5 4h11l-2 4 2 4H5" />);

export const DocsIcon = icon(
  "Docs",
  <path d="M3 5h6a3 3 0 0 1 3 3v12a2 2 0 0 0-2-2H3zM21 5h-6a3 3 0 0 0-3 3v12a2 2 0 0 1 2-2h7z" />,
);

export const SignOutIcon = icon(
  "SignOut",
  <path d="M9 4H5a1 1 0 0 0-1 1v14a1 1 0 0 0 1 1h4M16 16l4-4-4-4M20 12H9" />,
);

export const ClaimIcon = icon(
  "Claim",
  <path d="M12 3l7 3v6c0 4.5-3 7.5-7 9-4-1.5-7-4.5-7-9V6zM9 12l2 2 4-4" />,
);

export const CopyIcon = icon(
  "Copy",
  <>
    <rect x="8" y="8" width="12" height="12" rx="1.5" />
    <path d="M16 8V5a1 1 0 0 0-1-1H5a1 1 0 0 0-1 1v10a1 1 0 0 0 1 1h3" />
  </>,
);
