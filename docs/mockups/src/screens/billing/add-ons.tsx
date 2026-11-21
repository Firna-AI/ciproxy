// The monthly add-ons on Team and Business: extra starts and extra own
// machines. A state over a soft limit suggests the quantity that covers it.
// A person who cannot pay sees the saved quantities only.

import { Action } from "../../components/actions.tsx";
import { Quantity } from "../../components/figures.tsx";
import { DetailList, DetailRow, Panel } from "../../components/page.tsx";
import { formatCount, monthly } from "../../fixtures/format.ts";
import { addOnPrices, tiers } from "../../fixtures/tiers.ts";
import type { Usage, UsageScene } from "../../fixtures/usage.ts";

/** The add-on quantities that each state suggests to cover the use. */
const suggested: Partial<
  Record<UsageScene, { starts: number; machines: number }>
> = {
  "over-starts": { starts: 1, machines: 0 },
  "over-machines": { starts: 0, machines: 1 },
};

export function AddOns({
  scene,
  usage,
  canPay,
}: {
  scene: UsageScene;
  usage: Usage;
  canPay: boolean;
}) {
  const tier = tiers[usage.tier];
  if (!tier.addOns) {
    return (
      <Panel title="Add-ons">
        <p className="small muted">
          Team and Business can add starts and own machines.
        </p>
      </Panel>
    );
  }
  if (!canPay) return <SavedAddOns usage={usage} />;
  const quantities = suggested[scene] ?? {
    starts: usage.extraStarts,
    machines: usage.extraMachines,
  };
  const total =
    tier.price +
    quantities.starts * addOnPrices.startsPrice +
    quantities.machines * addOnPrices.machinePrice;
  const changed = suggested[scene] !== undefined;
  return (
    <Panel title="Add-ons">
      <div className="stack addons">
        <Quantity
          label="Extra starts"
          value={quantities.starts}
          caption={`${monthly(addOnPrices.startsPrice)} for each ${formatCount(addOnPrices.startsBlock)}`}
        />
        <Quantity
          label="Extra own machines"
          value={quantities.machines}
          caption={`${monthly(addOnPrices.machinePrice)} each`}
        />
        <div className="addons-total">
          <span className="stack-tight">
            <span className="strong">{monthly(total)}</span>
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

/** The saved add-on quantities and the monthly total, without controls. */
function SavedAddOns({ usage }: { usage: Usage }) {
  const tier = tiers[usage.tier];
  const total =
    tier.price +
    usage.extraStarts * addOnPrices.startsPrice +
    usage.extraMachines * addOnPrices.machinePrice;
  return (
    <Panel title="Add-ons">
      <DetailList>
        <DetailRow label="Extra starts">{usage.extraStarts}</DetailRow>
        <DetailRow label="Extra own machines">{usage.extraMachines}</DetailRow>
        <DetailRow label="Your plan with add-ons">{monthly(total)}</DetailRow>
      </DetailList>
    </Panel>
  );
}
