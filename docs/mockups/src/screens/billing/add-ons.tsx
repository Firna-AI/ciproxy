// The monthly add-ons on Team and Business: extra starts and extra own nodes.
// A state over a soft limit suggests the quantity that covers the use.

import { Action } from "../../components/actions.tsx";
import { Quantity } from "../../components/figures.tsx";
import { Panel } from "../../components/page.tsx";
import { formatMoney } from "../../fixtures/format.ts";
import { addOnPrices, tiers } from "../../fixtures/tiers.ts";
import type { Usage, UsageScene } from "../../fixtures/usage.ts";

/** The add-on quantities that each state suggests to cover the use. */
const suggested: Partial<
  Record<UsageScene, { starts: number; nodes: number }>
> = {
  "over-starts": { starts: 1, nodes: 0 },
  "over-nodes": { starts: 0, nodes: 1 },
};

export function AddOns({ scene, usage }: { scene: UsageScene; usage: Usage }) {
  const tier = tiers[usage.tier];
  if (!tier.addOns) {
    return (
      <Panel title="Add-ons">
        <p className="small muted">
          Team and Business can add starts and own nodes.
        </p>
      </Panel>
    );
  }
  const quantities = suggested[scene] ?? {
    starts: usage.extraStarts,
    nodes: usage.extraNodes,
  };
  const total =
    tier.price +
    quantities.starts * addOnPrices.startsPrice +
    quantities.nodes * addOnPrices.nodePrice;
  const changed = suggested[scene] !== undefined;
  return (
    <Panel title="Add-ons">
      <div className="stack addons">
        <Quantity
          label="Extra starts"
          value={quantities.starts}
          caption="$10 a month for each 10,000"
        />
        <Quantity
          label="Extra own nodes"
          value={quantities.nodes}
          caption="$20 a month each"
        />
        <div className="addons-total">
          <span className="stack-tight">
            <span className="strong">{formatMoney(total)} a month</span>
            <span className="caption">
              {changed
                ? "Stripe charges the difference for the rest of this period."
                : "Your plan with add-ons"}
            </span>
          </span>
          <Action tone={changed ? "primary" : "secondary"} disabled={!changed}>
            Save add-ons
          </Action>
        </div>
      </div>
    </Panel>
  );
}
