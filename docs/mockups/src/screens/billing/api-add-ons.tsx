// The add-ons of Plan and usage as the current API prices them. The API
// prices two choices on this page: the saved add-ons, and the add-ons that
// cover the use. Any other quantity has no price, so its total is not
// available and Save add-ons stays off. Use the add-ons that cover your use
// sets the offer. After Save add-ons, the panel shows what followed: a
// payment to finish on its Stripe page, a card that failed, or a result that
// the page did not get. While that payment waits, nothing else saves.

import { Action } from "../../components/actions.tsx";
import { Quantity } from "../../components/figures.tsx";
import { Panel } from "../../components/page.tsx";
import {
  apiAddOnCaptions,
  apiOffer,
  choiceCapacity,
  choiceMonthly,
  choiceName,
  sameCount,
  type AddOnCount,
  type ApiOffer,
} from "../../fixtures/current-api.ts";
import { tiers } from "../../fixtures/tiers.ts";
import { usageScenes, type UsageScene } from "../../fixtures/usage.ts";
import { ApiOutcomeNotice, type ApiOutcome } from "./api-outcome.tsx";

/** What followed Save add-ons. */
export type AddOnsOutcome = Extract<ApiOutcome, "invoice" | "card" | "unknown">;

function quoteText(saved: ApiOffer, offer: ApiOffer, shown: AddOnCount) {
  if (sameCount(shown, saved.addOns)) {
    return {
      total: choiceMonthly(saved),
      detail: `Your plan with add-ons: ${choiceCapacity(saved)}.`,
    };
  }
  if (sameCount(shown, offer.addOns)) {
    return {
      total: choiceMonthly(offer),
      detail: `${choiceCapacity(offer)}. Stripe charges the difference for the rest of this period.`,
    };
  }
  return {
    total: "Monthly total not available",
    detail:
      "Plan and usage prices the saved add-ons and the add-ons that cover your use.",
  };
}

export function ApiAddOns({
  scene,
  edited,
  outcome,
  waiting,
}: {
  scene: UsageScene;
  /** The quantities that the payer set, when not the saved ones. */
  edited?: AddOnCount;
  outcome?: AddOnsOutcome;
  /** A payment waits, so Save add-ons stays off. */
  waiting: boolean;
}) {
  const usage = usageScenes[scene];
  if (!tiers[usage.tier].addOns) {
    return (
      <Panel title="Add-ons">
        <p className="small muted">
          Team and Business can add starts and own machines.
        </p>
      </Panel>
    );
  }
  const saved: ApiOffer = {
    tier: usage.tier,
    addOns: { starts: usage.extraStarts, machines: usage.extraMachines },
  };
  const offer = apiOffer(scene);
  const shown = edited ?? saved.addOns;
  const suggests = !sameCount(offer.addOns, saved.addOns);
  const offerShown = suggests && sameCount(shown, offer.addOns);
  const words = quoteText(saved, offer, shown);
  return (
    <Panel title="Add-ons">
      <div className="stack addons">
        <Quantity
          label="Extra starts"
          value={shown.starts}
          caption={apiAddOnCaptions.starts}
        />
        <Quantity
          label="Extra own machines"
          value={shown.machines}
          caption={apiAddOnCaptions.machines}
        />
        <div className="addons-total addons-total-wrap">
          <span className="stack-tight">
            <span className="strong">{words.total}</span>
            <span className="caption">{words.detail}</span>
          </span>
          <span className="row-wrap">
            {suggests && !offerShown ? (
              <Action tone="ghost">Use the add-ons that cover your use</Action>
            ) : null}
            <Action
              tone={offerShown ? "primary" : "secondary"}
              disabled={!offerShown || waiting}
            >
              Save add-ons
            </Action>
          </span>
        </div>
        {outcome ? (
          <ApiOutcomeNotice outcome={outcome} choice={choiceName(offer)} />
        ) : null}
      </div>
    </Panel>
  );
}
