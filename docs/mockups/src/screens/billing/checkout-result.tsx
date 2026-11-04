// The return from Stripe Checkout: while ciproxy confirms the payment, a
// payment that Stripe still confirms after the wait, the confirmed plan, and
// the return after a payment that did not finish. Checkout takes cards only,
// and the page never asks for a second payment, as design call 38 says.

import { CircleCheck, Clock, CreditCard } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { LinkAction } from "../../components/actions.tsx";
import { Busy, StateBlock } from "../../components/feedback.tsx";
import { Meter } from "../../components/figures.tsx";
import { Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { AppShell } from "../../components/shell/app-shell.tsx";
import { formatCount } from "../../fixtures/format.ts";
import { workspace } from "../../fixtures/org.ts";
import { ownMachines, startsText, tiers } from "../../fixtures/tiers.ts";
import { firstTeamPeriod } from "../../fixtures/usage.ts";
import "./billing.css";

export const resultStates = [
  "confirming",
  "still-confirming",
  "confirmed",
  "cancelled",
] as const;
export type ResultState = (typeof resultStates)[number];

export function CheckoutResultScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: ResultState;
}) {
  return (
    <AppShell
      viewport={viewport}
      tier={state === "confirmed" ? "team" : "free"}
      active="billing"
    >
      <Panel>
        <div className="result">
          <ResultBody state={state} />
        </div>
      </Panel>
    </AppShell>
  );
}

function ResultBody({ state }: { state: ResultState }) {
  if (state === "confirming") {
    return (
      <div className="state-block">
        <Busy size={24} />
        <h2 className="state-title">Confirming your payment</h2>
        <p className="state-body">
          Stripe is confirming the payment. This takes a few seconds.
        </p>
      </div>
    );
  }
  if (state === "still-confirming") {
    return (
      <StateBlock icon={Clock} title="Stripe is still confirming your payment">
        {workspace.name} moves to Team when Stripe confirms it. This page
        updates by itself, and jobs keep running.
      </StateBlock>
    );
  }
  if (state === "cancelled") {
    return (
      <StateBlock
        icon={CreditCard}
        tone="warning"
        title="Your plan did not change"
        action={
          <>
            <LinkAction to={paths.checkoutStripe} tone="primary">
              Try again
            </LinkAction>
            <LinkAction to={paths.plans}>Back to plans</LinkAction>
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
        title={`${workspace.name} is on Team`}
        action={
          <>
            <LinkAction to={paths.addMachine} tone="primary">
              Add machines
            </LinkAction>
            <LinkAction to={paths.billing}>Open plan and usage</LinkAction>
          </>
        }
      >
        {startsText(team.starts)} a month and {ownMachines(team.machines)}.
        Stripe emailed the receipt to cal@mokly.ai.
      </StateBlock>
      <div className="result-meter">
        <Meter
          label="Starts this period"
          used={firstTeamPeriod.starts}
          max={team.starts}
          value={`${formatCount(firstTeamPeriod.starts)} of ${formatCount(team.starts)}`}
          detail={`The Team period runs ${firstTeamPeriod.period}.`}
        />
      </div>
    </>
  );
}
