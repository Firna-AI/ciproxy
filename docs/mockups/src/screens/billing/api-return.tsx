// The return from Stripe Checkout as the current API answers it. The API
// names no receipt address, so the confirmed plan says only that Stripe
// emails the receipt. A Checkout that is still open continues on Stripe. A
// payment that did not finish opens the plans again, because a new payment
// starts from a new review. Stripe returns to an address that names the
// workspace. A return without it, from an older address, uses the workspace
// that its tab saved, and a tab that saved none says so in place of a guess.

import { CircleCheck, CreditCard, ExternalLink, Info } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { LinkAction } from "../../components/actions.tsx";
import { StateBlock } from "../../components/feedback.tsx";
import { Meter } from "../../components/figures.tsx";
import { paths } from "../../components/paths.ts";
import { choiceCapacity } from "../../fixtures/current-api.ts";
import { formatCount } from "../../fixtures/format.ts";
import { workspace } from "../../fixtures/org.ts";
import { tiers } from "../../fixtures/tiers.ts";
import { firstTeamPeriod } from "../../fixtures/usage.ts";
import { AuthShell } from "../auth/auth-shell.tsx";

export const apiResultStates = ["api-confirmed", "open", "unfinished"] as const;
export type ApiResultState = (typeof apiResultStates)[number];

export function isApiResult(state: string): state is ApiResultState {
  return (apiResultStates as readonly string[]).includes(state);
}

export function ApiResultBody({ state }: { state: ApiResultState }) {
  if (state === "open") {
    return (
      <StateBlock
        icon={CreditCard}
        tone="warning"
        title="Your payment is not finished"
        action={
          <>
            <LinkAction
              to={paths.checkoutStripe}
              tone="primary"
              icon={ExternalLink}
            >
              Continue payment
            </LinkAction>
            <LinkAction to={paths.plans}>Back to plans</LinkAction>
          </>
        }
      >
        Stripe took no payment yet. {workspace.name} is still on Free, and jobs
        keep running.
      </StateBlock>
    );
  }
  if (state === "unfinished") {
    return (
      <StateBlock
        icon={CreditCard}
        tone="warning"
        title="Your plan did not change"
        action={
          <>
            <LinkAction to={paths.plans} tone="primary">
              Try again
            </LinkAction>
            <LinkAction to={paths.usageFreeOver}>
              Open plan and usage
            </LinkAction>
          </>
        }
      >
        Stripe took no payment. {workspace.name} is still on Free, and jobs keep
        running.
      </StateBlock>
    );
  }
  const team = tiers.team;
  return (
    <>
      <StateBlock
        icon={CircleCheck}
        tone="success"
        title={`${workspace.name} is on ${team.name}`}
        action={
          <>
            <LinkAction to={paths.addMachine} tone="primary">
              Add machines
            </LinkAction>
            <LinkAction to={paths.billing}>Open plan and usage</LinkAction>
          </>
        }
      >
        {choiceCapacity({ tier: "team", addOns: { starts: 0, machines: 0 } })}.
        Stripe emails the receipt.
      </StateBlock>
      <div className="result-meter">
        <Meter
          label="Starts this period"
          used={firstTeamPeriod.starts}
          max={team.starts}
          value={`${formatCount(firstTeamPeriod.starts)} of ${formatCount(team.starts)}`}
          detail={`The ${team.name} period runs ${firstTeamPeriod.period}.`}
        />
      </div>
    </>
  );
}

/** A Stripe return with no workspace, in a tab that saved none. */
export function ReturnWithoutWorkspace({ viewport }: { viewport: Viewport }) {
  return (
    <AuthShell viewport={viewport}>
      <StateBlock
        icon={Info}
        title="This tab cannot tell the workspace of this payment"
        action={
          <LinkAction to={paths.jobs} tone="primary">
            Open ciproxy
          </LinkAction>
        }
      >
        Stripe also tells ciproxy about the payment, so Plan and usage of its
        workspace shows the plan once Stripe confirms it.
      </StateBlock>
    </AuthShell>
  );
}
