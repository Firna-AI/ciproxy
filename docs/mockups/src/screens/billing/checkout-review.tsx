// The last ciproxy page before a plan changes: the tier, the add-on
// quantities, and the monthly total. A move from Free goes on to Stripe
// Checkout. A move between paid tiers takes effect at once and keeps the
// period: Stripe charges the prorated difference to the card on file, and the
// starts so far count toward the new tier.

import { ExternalLink } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { Action, LinkAction } from "../../components/actions.tsx";
import { Quantity } from "../../components/figures.tsx";
import {
  DetailList,
  DetailRow,
  PageHeader,
  Panel,
} from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { AppShell } from "../../components/shell/app-shell.tsx";
import { formatCount, formatMoney, monthly } from "../../fixtures/format.ts";
import { org } from "../../fixtures/org.ts";
import {
  addOnPrices,
  ownMachines,
  startsText,
  tiers,
} from "../../fixtures/tiers.ts";
import { businessMove, proratedCharge } from "../../fixtures/usage.ts";
import "./billing.css";

export type ReviewMove = "to-team" | "to-business";

const { team, business } = tiers;

const reviews = {
  "to-team": {
    from: "free",
    tier: team,
    fit: "Your use this month fits Team without add-ons.",
    today: team.price,
    back: paths.plans,
  },
  "to-business": {
    from: "team",
    tier: business,
    fit: "Your use this period fits Business without add-ons.",
    today: proratedCharge(team.price, business.price),
    back: paths.plansOnTeam,
  },
} as const;

function ChargeNote({ move }: { move: ReviewMove }) {
  if (move === "to-team") {
    return (
      <p className="caption">
        Stripe charges {formatMoney(team.price)} today, then on the 4th of each
        month. You pay on Stripe; ciproxy never sees your card.
      </p>
    );
  }
  return (
    <p className="caption">
      Business starts now. Stripe charges the card on file{" "}
      {formatMoney(reviews["to-business"].today)} for the rest of this period,
      then {formatMoney(business.price)} on the 4th of each month. The{" "}
      {formatCount(businessMove.starts)} starts of this period count toward
      Business.
    </p>
  );
}

export function CheckoutReviewScreen({
  viewport,
  move,
}: {
  viewport: Viewport;
  move: ReviewMove;
}) {
  const review = reviews[move];
  const { tier } = review;
  return (
    <AppShell viewport={viewport} tier={review.from} active="billing">
      <PageHeader
        crumbs={[{ label: "Plans", to: review.back }]}
        title={`Move to ${tier.name}`}
        subtitle={`For ${org.login}: ${startsText(tier.starts)} a month and ${ownMachines(tier.machines)}.`}
      />
      <div className="checkout">
        <Panel title="Add-ons">
          <div className="stack">
            <p className="small muted">{review.fit}</p>
            <Quantity
              label="Extra starts"
              value={0}
              caption={`${monthly(addOnPrices.startsPrice)} for each ${formatCount(addOnPrices.startsBlock)}`}
            />
            <Quantity
              label="Extra own machines"
              value={0}
              caption={`${monthly(addOnPrices.machinePrice)} each`}
            />
          </div>
        </Panel>
        <Panel title="Summary">
          <div className="stack">
            <DetailList>
              <DetailRow label={tier.name}>{formatMoney(tier.price)}</DetailRow>
              <DetailRow label="Extra starts · 0">{formatMoney(0)}</DetailRow>
              <DetailRow label="Extra own machines · 0">
                {formatMoney(0)}
              </DetailRow>
              <DetailRow label="Total a month">
                <strong>{formatMoney(tier.price)}</strong>
              </DetailRow>
              <DetailRow label="Charged today">
                {formatMoney(review.today)}
              </DetailRow>
            </DetailList>
            <ChargeNote move={move} />
            {move === "to-team" ? (
              <LinkAction
                to={paths.checkoutStripe}
                tone="primary"
                icon={ExternalLink}
                block
              >
                Continue to payment
              </LinkAction>
            ) : (
              <Action tone="primary" block>
                {`Move to ${tier.name}`}
              </Action>
            )}
            <LinkAction to={review.back} tone="ghost" block>
              Back to plans
            </LinkAction>
          </div>
        </Panel>
      </div>
    </AppShell>
  );
}
