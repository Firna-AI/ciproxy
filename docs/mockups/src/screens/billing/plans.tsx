// Compare the three tiers and choose one. The tier that covers the recent use
// carries a tag; the current tier cannot be chosen again.

import { Check } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { Action, LinkAction } from "../../components/actions.tsx";
import { PageHeader } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { AppShell } from "../../components/shell/app-shell.tsx";
import { Tag } from "../../components/status.tsx";
import { formatCount, formatMoney, monthly } from "../../fixtures/format.ts";
import type { TierKey } from "../../fixtures/org.ts";
import {
  addOnPrices,
  ownMachines,
  startsText,
  tierFeatures,
  tierOrder,
  tiers,
  type Tier,
} from "../../fixtures/tiers.ts";
import { freePeriodEnd, usageScenes } from "../../fixtures/usage.ts";
import "./billing.css";

export const plansStates = ["from-free", "on-team"] as const;
export type PlansState = (typeof plansStates)[number];

const freeUse = usageScenes["free-over"];

const context: Record<
  PlansState,
  { current: TierKey; covers: TierKey; use: string }
> = {
  "from-free": {
    current: "free",
    covers: "team",
    use: `This month so far: ${formatCount(freeUse.starts)} starts on ${ownMachines(freeUse.machines)}. At this pace, about ${formatCount(freeUse.projected)} starts by ${freePeriodEnd}.`,
  },
  "on-team": {
    current: "team",
    covers: "team",
    use: `Your last 30 days: ${formatCount(17_772)} starts on ${ownMachines(3)}.`,
  },
};

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
  const list = tierFeatures(tier);
  return (
    <section className={`tier-card${covers ? " is-recommended" : ""}`}>
      <div className="spread">
        <h2 className="tier-name">{tier.name}</h2>
        {current ? (
          <Tag>Current plan</Tag>
        ) : covers ? (
          <Tag tone="accent">Covers your use</Tag>
        ) : null}
      </div>
      <p className="tier-price">
        <span className="tier-amount">{formatMoney(tier.price)}</span>
        <span className="muted"> a month</span>
      </p>
      <p className="small">
        <strong>{formatCount(tier.starts)}</strong> starts a month ·{" "}
        <strong>{tier.machines}</strong> own{" "}
        {tier.machines === 1 ? "machine" : "machines"}
      </p>
      <TierAction tier={tier} current={current} covers={covers} />
      <div className="stack-tight">
        <p className="caption">{list.lead}</p>
        <ul className="tier-features">
          {list.items.map((item) => (
            <li key={item}>
              <Check size={14} className="tier-check" />
              <span className="small">{item}</span>
            </li>
          ))}
        </ul>
      </div>
      {tier.addOns ? (
        <p className="caption tier-addons">
          Add {startsText(addOnPrices.startsBlock)} for{" "}
          {monthly(addOnPrices.startsPrice)}, or an extra own machine for{" "}
          {monthly(addOnPrices.machinePrice)}.
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
