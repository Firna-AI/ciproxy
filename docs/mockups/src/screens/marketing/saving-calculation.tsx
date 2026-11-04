// The answer of the pricing question that the cost caption of the landing
// page links to: how the claim counts the saving. Every number comes from
// fixtures/cost-claim.ts, with the day on which each price was checked.

import { costClaim } from "../../fixtures/cost-claim.ts";
import {
  formatCount,
  formatMoney,
  formatMoneyCents,
  formatRate,
  unbroken,
} from "../../fixtures/format.ts";
import { ax42 } from "../../fixtures/hardware.ts";
import { planChoiceText } from "../../fixtures/plan-choice.ts";
import { listPrices } from "../../fixtures/price-lists.ts";
import { sizePhrase } from "../../fixtures/sizes.ts";
import "./saving-calculation.css";

/** The id of the question, which the cost caption opens. */
export const savingId = "saving";

/** The question of the pricing page that holds the calculation. */
export const savingQuestion = "How do you calculate the saving?";

const {
  machine,
  slots,
  slotSize,
  jobMinutes,
  starts,
  plan,
  withCiproxy,
  alternatives,
  cheaper,
  upToPercent,
} = costClaim;

const minutes = formatCount(jobMinutes);

export function SavingCalculation() {
  return (
    <div className="saving">
      <p>
        One new {machine} at its list price runs jobs in every minute of a{" "}
        {costClaim.monthDays}-day month. After a quarter for the host, its{" "}
        {ax42.threads} threads and {ax42.memoryGb} GB give {slots} slots of{" "}
        {sizePhrase(slotSize)}, so it runs {minutes} job minutes. At{" "}
        {unbroken(`${costClaim.averageJobMinutes} minutes`)} a job, that is{" "}
        {formatCount(starts)} starts, which {planChoiceText(plan)} covers.
      </p>
      <dl className="saving-table">
        <div className="saving-row">
          <dt>{machine}, list price</dt>
          <dd className="num">{formatMoneyCents(listPrices.machine.amount)}</dd>
        </div>
        <div className="saving-row">
          <dt>ciproxy {planChoiceText(plan)}</dt>
          <dd className="num">{formatMoneyCents(plan.price)}</dd>
        </div>
        <div className="saving-row is-total">
          <dt>Total with ciproxy</dt>
          <dd className="num">{formatMoneyCents(withCiproxy)}</dd>
        </div>
        {alternatives.map((alternative) => (
          <div key={alternative.name} className="saving-row">
            <dt>
              {alternative.runners}, {minutes} minutes at{" "}
              {formatRate(alternative.price.amount)}
            </dt>
            <dd className="num">{formatMoneyCents(alternative.cost)}</dd>
          </div>
        ))}
      </dl>
      <p>
        That is {upToPercent}% less than {cheaper.runners}, the cheaper of the
        two.
      </p>
      <p className="caption">
        The list prices come from the pages of Hetzner, Blacksmith, and GitHub
        on {unbroken(listPrices.machine.checked)}, without tax. Both hosted
        runners have {unbroken(`${slotSize.vcpu} vCPU`)}, as one slot does. The{" "}
        {machine} also costs {formatMoney(listPrices.machineSetup.amount)} once
        to set up. Free minutes of the hosted runners are not subtracted.
      </p>
    </div>
  );
}
