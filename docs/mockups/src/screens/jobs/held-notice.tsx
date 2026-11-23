// The notice of a label whose order has no place that can take its jobs now.
// As the design call of 23 November 2026 says, it gives the reason of the
// first place of the order that fits the size, such as "Overflow providers
// are off" or the spend limit of that provider. The reason comes from the
// placement of the label, as the label list shows it. An Admin gets the
// setting that clears the reason, when one does; everyone else reads Ask an
// Admin.

import { LinkAction } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { capitalized, holdText } from "../../components/hold-text.ts";
import { paths, type CataloguePath } from "../../components/paths.ts";
import { AskAdmin } from "../../components/role-actions.tsx";
import type { Role } from "../../fixtures/members.ts";
import {
  placementOf,
  type PlaceContext,
  type ProviderHold,
} from "../../fixtures/placement.ts";
import type { Label } from "../../fixtures/settings.ts";

type HoldAction = { label: string; to: CataloguePath };

/** The setting that clears a hold, when an Admin clears it in ciproxy. */
const holdActions: Readonly<Partial<Record<ProviderHold, HoldAction>>> = {
  off: { label: "Turn on overflow", to: paths.overflow },
  "not-connected": {
    label: "Connect a provider",
    to: paths.overflowNotConnected,
  },
  limit: { label: "Raise the limit", to: paths.overflowLimitReached },
  rejected: { label: "Connect a new key", to: paths.overflowBoatRejected },
};

function jobsWord(count: number): string {
  return count === 1 ? "1 job" : `${count} jobs`;
}

export function HeldNotice({
  label,
  context,
  count,
  role,
}: {
  label: Label;
  context: PlaceContext;
  /** The waiting jobs of the label. */
  count: number;
  role: Role;
}) {
  const placement = placementOf(label, context);
  if (placement.kind !== "waits" || placement.reason.kind !== "held") {
    return null;
  }
  const { provider, hold } = placement.reason;
  const action = holdActions[hold];
  return (
    <Notice
      tone="warning"
      title={`${jobsWord(count)} with the label ${label.name} wait`}
      actions={
        action && role === "admin" ? (
          <LinkAction to={action.to} size="sm">
            {action.label}
          </LinkAction>
        ) : action ? (
          <AskAdmin />
        ) : undefined
      }
    >
      {capitalized(holdText(provider, hold))}. No other place of its order can
      take them now.
    </Notice>
  );
}
