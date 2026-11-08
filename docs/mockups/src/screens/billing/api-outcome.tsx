// What follows a payment on the page that sent it, as the current API allows
// the page to know it. A payment that Stripe has not finished opens its
// Stripe page. A card that failed opens the billing portal. When the page did
// not get the result, it says so, and the Plan and usage read after it can
// prove only two things: the choice is the current plan, or a payment or a
// billing change waits. The page never says that such a payment charged or
// did not charge.

import { ExternalLink } from "lucide-react";

import { Action, LinkAction } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { paths } from "../../components/paths.ts";

/** The result of a payment that the page shows. */
export type ApiOutcome = "invoice" | "card" | "unknown" | "current" | "waits";

const openUsage = (
  <LinkAction to={paths.billing} size="sm">
    Open plan and usage
  </LinkAction>
);

export function ApiOutcomeNotice({
  outcome,
  choice,
}: {
  outcome: ApiOutcome;
  /** The submitted choice, such as "Team with 1 block of starts". */
  choice: string;
}) {
  if (outcome === "invoice") {
    return (
      <Notice
        tone="warning"
        title="Your payment is not finished"
        actions={
          <>
            <Action tone="primary" size="sm" icon={ExternalLink}>
              Open the Stripe page
            </Action>
            <Action size="sm">Check again</Action>
          </>
        }
      >
        Finish it on its Stripe page. Your plan changes when Stripe confirms the
        payment.
      </Notice>
    );
  }
  if (outcome === "card") {
    return (
      <Notice
        tone="danger"
        title="Stripe could not charge your card"
        actions={
          <>
            <Action tone="primary" size="sm" icon={ExternalLink}>
              Update card
            </Action>
            <Action size="sm">Try again</Action>
          </>
        }
      >
        Your plan stays as it was, and jobs keep running. Update the card, then
        try again.
      </Notice>
    );
  }
  if (outcome === "current") {
    return (
      <Notice
        tone="info"
        title={`${choice} is your current plan`}
        actions={openUsage}
      >
        Plan and usage shows it.
      </Notice>
    );
  }
  if (outcome === "waits") {
    return (
      <Notice
        tone="warning"
        title="A payment or a billing change is in progress"
        actions={openUsage}
      >
        Plan and usage shows it until it ends. Jobs keep running.
      </Notice>
    );
  }
  return (
    <Notice
      tone="warning"
      title="Unable to confirm this payment"
      actions={
        <>
          {openUsage}
          <Action size="sm">Try again</Action>
        </>
      }
    >
      ciproxy did not get the result. Plan and usage shows the current plan.
      Jobs keep running.
    </Notice>
  );
}
