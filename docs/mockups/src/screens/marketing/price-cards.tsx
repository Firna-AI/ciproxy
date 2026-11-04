// The three tiers of the pricing page as cells: the price, the limits, the
// action, and what each tier includes or adds. Every action says Start free
// and opens sign-in, because every workspace starts on Free and picks a paid
// tier in the dashboard.

import { LinkAction } from "../../components/actions.tsx";
import { paths } from "../../components/paths.ts";
import { TierFeatureList } from "../../components/tier-feature-list.tsx";
import { formatCount, formatMoney } from "../../fixtures/format.ts";
import type { TierKey } from "../../fixtures/org.ts";
import {
  tierFeatures,
  tierOrder,
  tiers,
  type Tier,
} from "../../fixtures/tiers.ts";
import { Cells } from "./section.tsx";
import "./price-cards.css";

const blurbs: Readonly<Record<TierKey, string>> = {
  free: "Try ciproxy on one machine.",
  team: "For daily CI across your GitHub organizations.",
  business: `For large fleets, with ${tiers.business.history} of job history.`,
};

function PriceCard({ tier }: { tier: Tier }) {
  const list = tierFeatures(tier);
  const team = tier.key === "team";
  return (
    <div className="mk-cell price-card">
      <div className="stack-tight">
        <h2 className="price-name">{tier.name}</h2>
        <p className="price-blurb">{blurbs[tier.key]}</p>
      </div>
      <p className="price-amount-row">
        <span className="price-amount num">{formatMoney(tier.price)}</span>
        <span className="muted">a month</span>
      </p>
      <ul className="price-limits">
        <li>
          <strong className="num">{formatCount(tier.starts)}</strong> starts a
          month
        </li>
        <li>
          <strong className="num">{tier.machines}</strong> own{" "}
          {tier.machines === 1 ? "machine" : "machines"}
        </li>
      </ul>
      <LinkAction to={paths.signIn} tone={team ? "primary" : "secondary"} block>
        Start free
      </LinkAction>
      <div className="price-features">
        <p className="caption">{list.lead}</p>
        <TierFeatureList items={list.items} />
      </div>
    </div>
  );
}

export function PriceCards() {
  return (
    <Cells columns={3} className="price-cells">
      {tierOrder.map((key) => (
        <PriceCard key={key} tier={tiers[key]} />
      ))}
    </Cells>
  );
}
