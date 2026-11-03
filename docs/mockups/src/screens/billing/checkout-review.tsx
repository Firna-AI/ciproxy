// The last ciproxy page before Stripe Checkout: the tier, the add-on
// quantities, and the monthly total.

import { ExternalLink } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { LinkAction } from "../../components/actions.tsx";
import { Quantity } from "../../components/figures.tsx";
import {
  DetailList,
  DetailRow,
  PageHeader,
  Panel,
} from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { AppShell } from "../../components/shell/app-shell.tsx";
import { formatCount, formatMoneyCents } from "../../fixtures/format.ts";
import { org } from "../../fixtures/org.ts";
import { tiers } from "../../fixtures/tiers.ts";
import "./billing.css";

export function CheckoutReviewScreen({ viewport }: { viewport: Viewport }) {
  const team = tiers.team;
  return (
    <AppShell viewport={viewport} tier="free" active="billing">
      <PageHeader
        crumbs={[{ label: "Plans", to: paths.plans }]}
        title="Move to Team"
        subtitle={`For ${org.login}: ${formatCount(team.starts)} starts and ${team.nodes} own nodes a month.`}
      />
      <div className="checkout">
        <Panel title="Add-ons">
          <div className="stack">
            <p className="small muted">
              Your use this month fits Team without add-ons.
            </p>
            <Quantity
              label="Extra starts"
              value={0}
              caption="$10 a month for each 10,000"
            />
            <Quantity
              label="Extra own nodes"
              value={0}
              caption="$20 a month each"
            />
          </div>
        </Panel>
        <Panel title="Summary">
          <div className="stack">
            <DetailList>
              <DetailRow label="Team">{formatMoneyCents(team.price)}</DetailRow>
              <DetailRow label="Extra starts · 0">
                {formatMoneyCents(0)}
              </DetailRow>
              <DetailRow label="Extra own nodes · 0">
                {formatMoneyCents(0)}
              </DetailRow>
              <DetailRow label="Total a month">
                <strong>{formatMoneyCents(team.price)}</strong>
              </DetailRow>
            </DetailList>
            <p className="caption">
              Stripe charges {formatMoneyCents(team.price)} today, then on the
              4th of each month. You pay on Stripe; ciproxy never sees your
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
            <LinkAction to={paths.plans} tone="ghost" block>
              Back to plans
            </LinkAction>
          </div>
        </Panel>
      </div>
    </AppShell>
  );
}
