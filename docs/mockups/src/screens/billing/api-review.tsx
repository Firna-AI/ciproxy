// The summary of a review as the current API prices it: the plan, the add-on
// quantities without line amounts, the monthly total, and what Stripe
// charges today. From Free, the payment goes on to Stripe Checkout. Between
// paid plans, the server stores the quote of each review, and the payment
// names it, so Stripe charges exactly the reviewed amount. When the API
// cannot price the choice, the summary says so and offers Try again. When the
// API refuses the quote at payment, the summary asks for a new review. When a
// payment cannot start, the summary says why and offers what recovers it: a
// Plan and usage read that failed, a read that still runs, a payment that
// waits, or a person who cannot pay. None of these states has a button that
// pays.

import type { ReactNode } from "react";
import { ExternalLink } from "lucide-react";

import { Action, LinkAction } from "../../components/actions.tsx";
import { Busy } from "../../components/feedback.tsx";
import { DetailList, DetailRow, Panel } from "../../components/page.tsx";
import { paths, type CataloguePath } from "../../components/paths.ts";
import { formatMoney, monthly } from "../../fixtures/format.ts";
import type { Tier } from "../../fixtures/tiers.ts";
import { teamPeriodEnd } from "../../fixtures/usage.ts";

/**
 * A review as the current API gives it: priced for Checkout from Free,
 * priced for a move between paid plans, not priced, or refused at payment
 * because its quote is unknown, already used, or out of date. Or a payment
 * that cannot start: the Plan and usage read failed or still runs, a payment
 * or a billing change waits, or the person cannot pay.
 */
export type ApiSummaryState =
  | "checkout"
  | "paid"
  | "unpriced"
  | "refused"
  | "status-failed"
  | "checking"
  | "waiting"
  | "not-payer";

/** The note and the button of a review that the API priced. */
function PricedPay({
  tier,
  today,
  paid,
  payOff,
}: {
  tier: Tier;
  today: number;
  /** A move between paid plans, which charges the card on file at once. */
  paid: boolean;
  /** A payment of this review failed, so the button stays off. */
  payOff: boolean;
}) {
  if (!paid) {
    return (
      <>
        <p className="caption">
          Stripe charges {formatMoney(tier.price)} today, then on the 4th of
          each month. You pay by card on Stripe, and ciproxy never sees your
          card.
        </p>
        <LinkAction
          to={paths.checkoutStripe}
          tone="primary"
          icon={ExternalLink}
          block
        >
          Continue to payment
        </LinkAction>
      </>
    );
  }
  return (
    <>
      <p className="caption">
        {tier.name} starts now. Stripe charges the card on file{" "}
        {formatMoney(today)} for the rest of this period, which ends on{" "}
        {teamPeriodEnd}, then {formatMoney(tier.price)} on the 4th of each
        month. The starts of this period count toward {tier.name}.
      </p>
      {payOff ? (
        <Action tone="primary" block disabled>
          {`Move to ${tier.name}`}
        </Action>
      ) : (
        <LinkAction to={paths.checkoutCurrentRefused} tone="primary" block>
          {`Move to ${tier.name}`}
        </LinkAction>
      )}
    </>
  );
}

/** A summary with no price: what stops the payment, and what recovers it. */
function SummaryBlock({
  title,
  body,
  action,
  back,
}: {
  title: string;
  body: string;
  action: ReactNode;
  back: ReactNode;
}) {
  return (
    <Panel title="Summary">
      <div className="stack">
        <div className="stack-tight">
          <p className="strong">{title}</p>
          <p className="small">{body}</p>
        </div>
        {action}
        {back}
      </div>
    </Panel>
  );
}

export function ApiSummary({
  tier,
  today,
  state,
  back,
  payOff = false,
}: {
  tier: Tier;
  /** What Stripe charges today, in dollars. */
  today: number;
  state: ApiSummaryState;
  /** The Plans screen that Back to plans opens. */
  back: CataloguePath;
  /** A payment of this review failed, so the priced button stays off. */
  payOff?: boolean;
}) {
  const backToPlans = (
    <LinkAction to={back} tone="ghost" block>
      Back to plans
    </LinkAction>
  );
  const block = (title: string, body: string, action: ReactNode) => (
    <SummaryBlock
      title={title}
      body={body}
      action={action}
      back={backToPlans}
    />
  );
  if (state === "unpriced") {
    return block(
      "Unable to price this change",
      "The price of this choice did not load. Try again.",
      <Action block>Try again</Action>,
    );
  }
  if (state === "refused") {
    return block(
      "This review is out of date",
      "The payment did not start. Review the change again to see its current charge.",
      <LinkAction to={paths.checkoutCurrentPaid} tone="primary" block>
        Review again
      </LinkAction>,
    );
  }
  if (state === "status-failed") {
    return block(
      "Unable to load plan and usage",
      "Payments wait until it loads. Jobs keep running.",
      <LinkAction to={paths.checkoutCurrentPaid} tone="primary" block>
        Try again
      </LinkAction>,
    );
  }
  if (state === "waiting") {
    return block(
      "A payment or a billing change is in progress",
      "Plan and usage shows it until it ends. Jobs keep running.",
      <LinkAction to={paths.billing} block>
        Open plan and usage
      </LinkAction>,
    );
  }
  if (state === "not-payer") {
    return block(
      "Only an Admin or a Billing person can pay",
      "Ask an Admin or a Billing person.",
      null,
    );
  }
  if (state === "checking") {
    return (
      <Panel title="Summary">
        <div className="stack">
          <div className="row small" role="status">
            <Busy />
            <span>Checking your plan</span>
          </div>
          {backToPlans}
        </div>
      </Panel>
    );
  }
  return (
    <Panel title="Summary">
      <div className="stack">
        <DetailList>
          <DetailRow label="Plan">{tier.name}</DetailRow>
          <DetailRow label="Extra starts">None</DetailRow>
          <DetailRow label="Extra own machines">None</DetailRow>
          <DetailRow label="Total a month">
            <strong>{monthly(tier.price)}</strong>
          </DetailRow>
          <DetailRow label="Charged today">{formatMoney(today)}</DetailRow>
        </DetailList>
        <PricedPay
          tier={tier}
          today={today}
          paid={state === "paid"}
          payOff={payOff}
        />
        {backToPlans}
      </div>
    </Panel>
  );
}
