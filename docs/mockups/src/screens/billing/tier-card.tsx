// One tier on the Plans page: the price, the limits, the action, and what the
// tier includes or adds, with "Coming later" under a feature that ships
// later. A person who cannot pay reads who can choose in place of the action.
// While a payment waits, the action stays off.

import { Action, LinkAction } from "../../components/actions.tsx";
import type { CataloguePath } from "../../components/paths.ts";
import { AskPayer } from "../../components/role-actions.tsx";
import { Tag } from "../../components/status.tsx";
import { TierFeatureList } from "../../components/tier-feature-list.tsx";
import { formatCount, formatMoney, monthly } from "../../fixtures/format.ts";
import {
  addOnPrices,
  startsText,
  tierFeatures,
  type Tier,
} from "../../fixtures/tiers.ts";

export type TierCardProps = {
  tier: Tier;
  current: boolean;
  covers: boolean;
  /** The screen that the action opens, where the catalogue has one. */
  to: CataloguePath | undefined;
  /** An Admin or a Billing person chooses; a Viewer reads who can. */
  canPay: boolean;
  /** A payment waits, so no tier can be chosen until it ends. */
  waiting?: boolean;
  /** The card names the prices of the add-ons. The current API gives none. */
  unitPrices?: boolean;
};

export function TierCard(props: TierCardProps) {
  const { tier, current, covers, unitPrices = true } = props;
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
      <TierAction {...props} />
      <div className="stack-tight">
        <p className="caption">{list.lead}</p>
        <TierFeatureList items={list.items} size="sm" />
      </div>
      {tier.addOns && unitPrices ? (
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
  to,
  canPay,
  waiting,
}: TierCardProps) {
  if (current) {
    return (
      <Action block disabled>
        Current plan
      </Action>
    );
  }
  if (!canPay) return <AskPayer />;
  const label = tier.key === "free" ? "Move to Free" : `Choose ${tier.name}`;
  const tone = covers ? "primary" : "secondary";
  if (waiting) {
    return (
      <Action tone={tone} block disabled>
        {label}
      </Action>
    );
  }
  return to ? (
    <LinkAction to={to} tone={tone} block>
      {label}
    </LinkAction>
  ) : (
    <Action tone={tone} block>
      {label}
    </Action>
  );
}
