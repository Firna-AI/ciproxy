// The summary of a review as the current API prices it: the plan, the add-on
// quantities without line amounts, the monthly total, and what Stripe
// charges today. From Free, the payment goes on to Stripe Checkout. Between
// paid plans, the server stores the quote of each review, and the payment
// names it, so Stripe charges exactly the reviewed amount. When the API
// cannot price the choice, the summary says so and offers Try again. When the
// API refuses the quote at payment, the summary asks for a new review.
// Neither of these two states has a button that pays.

import { ExternalLink } from "lucide-react";

import { Action, LinkAction } from "../../components/actions.tsx";
import { DetailList, DetailRow, Panel } from "../../components/page.tsx";
import { paths, type CataloguePath } from "../../components/paths.ts";
import { formatMoney, monthly } from "../../fixtures/format.ts";
import type { Tier } from "../../fixtures/tiers.ts";
import { teamPeriodEnd } from "../../fixtures/usage.ts";

/**
 * A review as the current API gives it: priced for Checkout from Free,
 * priced for a move between paid plans, not priced, or refused at payment
 * because its quote is unknown, already used, or out of date.
 */
export type ApiSummaryState = "checkout" | "paid" | "unpriced" | "refused";

/** The note and the button of a review that the API priced. */
function PricedPay({
  tier,
  today,
  paid,
}: {
  tier: Tier;
  today: number;
  /** A move between paid plans, which charges the card on file at once. */
  paid: boolean;
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
      <LinkAction to={paths.checkoutCurrentRefused} tone="primary" block>
        {`Move to ${tier.name}`}
      </LinkAction>
    </>
  );
}

export function ApiSummary({
  tier,
  today,
  state,
  back,
}: {
  tier: Tier;
  /** What Stripe charges today, in dollars. */
  today: number;
  state: ApiSummaryState;
  /** The Plans screen that Back to plans opens. */
  back: CataloguePath;
}) {
  const backToPlans = (
    <LinkAction to={back} tone="ghost" block>
      Back to plans
    </LinkAction>
  );
  if (state === "unpriced") {
    return (
      <Panel title="Summary">
        <div className="stack">
          <div className="stack-tight">
            <p className="strong">Unable to price this change</p>
            <p className="small">
              The price of this choice did not load. Try again.
            </p>
          </div>
          <Action block>Try again</Action>
          {backToPlans}
        </div>
      </Panel>
    );
  }
  if (state === "refused") {
    return (
      <Panel title="Summary">
        <div className="stack">
          <div className="stack-tight">
            <p className="strong">This review is out of date</p>
            <p className="small">
              The payment did not start. Review the change again to see its
              current charge.
            </p>
          </div>
          <LinkAction to={paths.checkoutCurrentPaid} tone="primary" block>
            Review again
          </LinkAction>
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
        <PricedPay tier={tier} today={today} paid={state === "paid"} />
        {backToPlans}
      </div>
    </Panel>
  );
}
