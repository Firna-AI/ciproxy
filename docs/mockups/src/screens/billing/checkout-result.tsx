// The return from Stripe Checkout: while ciproxy confirms the payment, the
// confirmed plan, and the return after a payment that did not finish.

import { CircleCheck, CreditCard } from "lucide-react";
import type { Viewport } from "@mokly/mokly";
import { Spinner } from "@firna/ui/spinner";

import { LinkAction } from "../../components/actions.tsx";
import { StateBlock } from "../../components/feedback.tsx";
import { Meter } from "../../components/figures.tsx";
import { Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { AppShell } from "../../components/shell/app-shell.tsx";
import { formatCount } from "../../fixtures/format.ts";
import { org } from "../../fixtures/org.ts";
import { tiers } from "../../fixtures/tiers.ts";
import { usageScenes } from "../../fixtures/usage.ts";
import { theme } from "../../theme/firna.tsx";
import "./billing.css";

export const resultStates = ["confirming", "confirmed", "cancelled"] as const;
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
        <Spinner size={28} color={theme.colors.primary} />
        <h2 className="state-title">Confirming your payment</h2>
        <p className="state-body">
          Stripe is confirming the payment. This takes a few seconds.
        </p>
      </div>
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
        Stripe took no payment. {org.login} is still on Free, and jobs keep
        running.
      </StateBlock>
    );
  }
  const used = usageScenes["free-over"].starts;
  return (
    <>
      <StateBlock
        icon={CircleCheck}
        title={`${org.login} is on Team`}
        action={
          <>
            <LinkAction to={paths.addMachine} tone="primary">
              Add machines
            </LinkAction>
            <LinkAction to={paths.billing}>Open plan and usage</LinkAction>
          </>
        }
      >
        {formatCount(tiers.team.starts)} starts and {tiers.team.nodes} own nodes
        a month. Stripe emailed the receipt to cal@mokly.ai.
      </StateBlock>
      <div className="result-meter">
        <Meter
          label="Starts this period"
          used={used}
          max={tiers.team.starts}
          value={`${formatCount(used)} of ${formatCount(tiers.team.starts)}`}
        />
      </div>
    </>
  );
}
