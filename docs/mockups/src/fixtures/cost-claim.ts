// The cost claim of the landing page, and its calculation on the pricing
// page. One new Hetzner AX42 at its list price, not from the server auction,
// runs jobs in every minute of a 30-day month. Its two disks form one
// filesystem, as the answer to item 75 says, so its slots and its job disk
// decide the default jobs at a time. ciproxy counts the machine and the
// cheapest plan that covers the starts. Blacksmith and GitHub-hosted runners
// count the same job minutes at their list prices per minute, for the same
// VM size. The prices and their sources are in price-lists.ts.

import { ax42, jobsAtOnce, machineSlots, slotSize } from "./hardware.ts";
import { cheapestPlan } from "./plan-choice.ts";
import { listPrices, type ListPrice } from "./price-lists.ts";
import { defaultLabelSize } from "./sizes.ts";

/** Money to whole cents, so a sum of per-minute prices stays exact. */
function cents(amount: number): number {
  return Math.round(amount * 100) / 100;
}

const averageJobMinutes = 5;
const monthDays = 30;
const slots = machineSlots(ax42);
const concurrentJobs = jobsAtOnce(ax42, slots, defaultLabelSize);
const jobMinutes = concurrentJobs * monthDays * 24 * 60;
const starts = Math.ceil(jobMinutes / averageJobMinutes);
const plan = cheapestPlan(starts, 1);
const withCiproxy = listPrices.machine.amount + plan.price;

/** The same job minutes on a hosted runner, at its price per minute. */
export type Alternative = {
  /** The short name, for a label such as the bar of a chart. */
  name: string;
  /** The name in a sentence, such as "less than GitHub-hosted runners". */
  runners: string;
  price: ListPrice;
  cost: number;
};

function alternative(
  name: string,
  runners: string,
  price: ListPrice,
): Alternative {
  return { name, runners, price, cost: cents(jobMinutes * price.amount) };
}

const alternatives: readonly Alternative[] = [
  alternative("Blacksmith", "Blacksmith", listPrices.blacksmith),
  alternative("GitHub-hosted", "GitHub-hosted runners", listPrices.github),
];

const cheaper = alternatives.reduce((best, next) =>
  next.cost < best.cost ? next : best,
);

export const costClaim = {
  machine: "Hetzner AX42",
  /** The average job length that the claim assumes, in minutes. */
  averageJobMinutes,
  monthDays,
  /** The CPU and memory slots of the AX42, before the disk limit. */
  slots,
  slotSize,
  /** The default jobs that fit both the slots and the job disk at a time. */
  concurrentJobs,
  /** The job minutes of every concurrent job in every minute of the month. */
  jobMinutes,
  starts,
  /** The cheapest tier with add-ons that covers the starts. */
  plan,
  /** The machine and the plan, a month. */
  withCiproxy,
  alternatives,
  /** The cheaper alternative, which the saving and its title compare. */
  cheaper,
  /** The saving against the cheaper alternative, down to a whole percent. */
  upToPercent: Math.floor((1 - withCiproxy / cheaper.cost) * 100),
  /** The dollars a month less than the cheaper alternative. */
  saved: cents(cheaper.cost - withCiproxy),
} as const;
