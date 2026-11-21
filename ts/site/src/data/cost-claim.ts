// The cost claim of the landing page, and its calculation on the pricing
// page. One new Hetzner AX42 at its list price, not from the server auction,
// runs jobs in every minute of a 30-day month. ciproxy counts the machine and
// the cheapest plan that covers the starts. Blacksmith and GitHub-hosted
// runners count the same job minutes at their list prices per minute, for the
// same VM size. The prices and their sources are in price-lists.ts.

import {
  ax42,
  machineConcurrentJobs,
  machineSlots,
  type Hardware,
} from "./hardware.ts";
import { cheapestPlan, type PlanChoice } from "./plan-choice.ts";
import { listPrices, type ListPrice } from "./price-lists.ts";

/** A hosted runner that the claim compares, at its price per minute. */
export type Runner = {
  /** The short name, for a label such as the bar of a chart. */
  name: string;
  /** The name in a sentence, such as "less than GitHub-hosted runners". */
  runners: string;
  price: ListPrice;
};

/** The same job minutes on a hosted runner, and their cost a month. */
export type Alternative = Runner & { cost: number };

/** What the claim assumes: one machine that runs jobs all month. */
export type ClaimBasis = {
  machine: string;
  hardware: Pick<Hardware, "threads" | "memoryGb" | "diskGb">;
  /** The list price of the machine, a month. */
  machinePrice: number;
  runners: readonly [Runner, ...Runner[]];
  /** The average job length, in minutes. */
  averageJobMinutes: number;
  monthDays: number;
};

export type CostClaim = {
  machine: string;
  averageJobMinutes: number;
  monthDays: number;
  /** The CPU and memory slots of the machine, before disk limits. */
  slots: number;
  /** The default jobs that fit both slots and job disk at the same time. */
  concurrentJobs: number;
  /** The minutes of every concurrent job in every minute of the month. */
  jobMinutes: number;
  starts: number;
  /** The cheapest tier with add-ons that covers the starts. */
  plan: PlanChoice;
  machinePrice: number;
  /** The machine and the plan, a month. */
  withCiproxy: number;
  alternatives: readonly Alternative[];
  /** The cheaper alternative, which the saving and its title compare. */
  cheaper: Alternative;
  /** The saving against the cheaper alternative, down to a whole percent. */
  upToPercent: number;
  /** The dollars a month less than the cheaper alternative. */
  saved: number;
};

/** Money to whole cents, so that a sum of prices per minute stays exact. */
function cents(amount: number): number {
  return Math.round(amount * 100) / 100;
}

/** The claim of one machine that runs jobs in every minute of a month. */
export function costClaimFor(basis: ClaimBasis): CostClaim {
  const slots = machineSlots(basis.hardware);
  const concurrentJobs = machineConcurrentJobs(basis.hardware);
  const jobMinutes = concurrentJobs * basis.monthDays * 24 * 60;
  const starts = Math.ceil(jobMinutes / basis.averageJobMinutes);
  const plan = cheapestPlan(starts, 1);
  const withCiproxy = basis.machinePrice + plan.price;
  const priced = (runner: Runner): Alternative => ({
    ...runner,
    cost: cents(jobMinutes * runner.price.amount),
  });
  const [firstRunner, ...otherRunners] = basis.runners;
  const first = priced(firstRunner);
  const rest = otherRunners.map(priced);
  const alternatives = [first, ...rest];
  const cheaper = rest.reduce(
    (best, next) => (next.cost < best.cost ? next : best),
    first,
  );
  return {
    machine: basis.machine,
    averageJobMinutes: basis.averageJobMinutes,
    monthDays: basis.monthDays,
    slots,
    concurrentJobs,
    jobMinutes,
    starts,
    plan,
    machinePrice: basis.machinePrice,
    withCiproxy,
    alternatives,
    cheaper,
    // Down, never up, so that the claim never says more than the numbers.
    upToPercent: Math.floor((1 - withCiproxy / cheaper.cost) * 100),
    saved: cents(cheaper.cost - withCiproxy),
  };
}

/** The id of the pricing question with the calculation of the claim. */
export const savingId = "saving";

/** The claim of the landing page, from the dated list prices. */
export const costClaim = costClaimFor({
  machine: "Hetzner AX42",
  hardware: ax42,
  machinePrice: listPrices.machine.amount,
  runners: [
    { name: "Blacksmith", runners: "Blacksmith", price: listPrices.blacksmith },
    {
      name: "GitHub-hosted",
      runners: "GitHub-hosted runners",
      price: listPrices.github,
    },
  ],
  averageJobMinutes: 5,
  monthDays: 30,
});
