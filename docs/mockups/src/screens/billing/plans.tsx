// Compare the three tiers and choose one. The tier that covers the recent use
// carries a badge; the current tier cannot be chosen again.

import { Check } from "lucide-react";
import type { Viewport } from "@mokly/mokly";
import { Badge } from "@firna/ui/badge";

import { Action, LinkAction } from "../../components/actions.tsx";
import { PageHeader } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { AppShell } from "../../components/shell/app-shell.tsx";
import { formatCount, formatMoney } from "../../fixtures/format.ts";
import type { TierKey } from "../../fixtures/org.ts";
import {
  everyTier,
  tierOrder,
  tiers,
  type Tier,
} from "../../fixtures/tiers.ts";
import "./billing.css";

export const plansStates = ["from-free", "on-team"] as const;
export type PlansState = (typeof plansStates)[number];

const context: Record<
  PlansState,
  { current: TierKey; covers: TierKey; use: string }
> = {
  "from-free": {
    current: "free",
    covers: "team",
    use: "This month so far: 2,214 starts on 1 own node. At this pace, about 18,200 starts by 30 November.",
  },
  "on-team": {
    current: "team",
    covers: "team",
    use: "Your last 30 days: 17,772 starts on 3 own nodes.",
  },
};

function features(tier: Tier): { lead: string; items: readonly string[] } {
  if (tier.key === "free")
    return { lead: "Every plan has", items: [...everyTier, ...tier.adds] };
  const below = tier.key === "team" ? "Free" : "Team";
  return { lead: `Everything in ${below}, plus`, items: tier.adds };
}

export function PlansScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: PlansState;
}) {
  const { current, covers, use } = context[state];
  return (
    <AppShell viewport={viewport} tier={current} active="billing">
      <PageHeader
        crumbs={[
          {
            label: "Plan and usage",
            to: current === "free" ? paths.usageFreeOver : paths.billing,
          },
        ]}
        title="Plans"
        subtitle={use}
      />
      <div className="tier-grid">
        {tierOrder.map((key) => (
          <TierCard
            key={key}
            tier={tiers[key]}
            current={key === current}
            covers={key === covers && key !== current}
          />
        ))}
      </div>
      <p className="caption">
        Limits are soft: jobs keep running past them, and ciproxy offers the
        plan or add-on that covers your use. Prices in US dollars, billed
        monthly through Stripe.
      </p>
    </AppShell>
  );
}

function TierCard({
  tier,
  current,
  covers,
}: {
  tier: Tier;
  current: boolean;
  covers: boolean;
}) {
  const list = features(tier);
  return (
    <section className={`tier-card${covers ? " is-recommended" : ""}`}>
      <div className="spread">
        <h2 className="tier-name">{tier.name}</h2>
        {current ? (
          <Badge tone="neutral" size="sm">
            Current plan
          </Badge>
        ) : covers ? (
          <Badge tone="primary" size="sm">
            Covers your use
          </Badge>
        ) : null}
      </div>
      <p className="tier-price">
        <span className="tier-amount">{formatMoney(tier.price)}</span>
        <span className="muted"> a month</span>
      </p>
      <p className="small">
        <strong>{formatCount(tier.starts)}</strong> starts a month ·{" "}
        <strong>{tier.nodes}</strong> own {tier.nodes === 1 ? "node" : "nodes"}
      </p>
      <TierAction tier={tier} current={current} covers={covers} />
      <div className="stack-tight">
        <p className="caption">{list.lead}</p>
        <ul className="tier-features">
          {list.items.map((item) => (
            <li key={item}>
              <Check size={15} className="state-passed" />
              <span className="small">{item}</span>
            </li>
          ))}
        </ul>
      </div>
      {tier.addOns ? (
        <p className="caption tier-addons">
          Add 10,000 starts for $10 or an own node for $20 a month.
        </p>
      ) : null}
    </section>
  );
}

function TierAction({
  tier,
  current,
  covers,
}: {
  tier: Tier;
  current: boolean;
  covers: boolean;
}) {
  if (current) {
    return (
      <Action block disabled>
        Current plan
      </Action>
    );
  }
  if (covers) {
    return (
      <LinkAction to={paths.checkoutReview} tone="primary" block>
        {`Choose ${tier.name}`}
      </LinkAction>
    );
  }
  return (
    <Action block>
      {tier.key === "free" ? "Move to Free" : `Choose ${tier.name}`}
    </Action>
  );
}
