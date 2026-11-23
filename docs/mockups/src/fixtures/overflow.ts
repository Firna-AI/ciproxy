// The overflow of mokly-ai: the workspace switch and the overflow wait, which
// hold for every provider, and the account of each connected provider, with
// its own monthly spend limit. mokly-ai connects Boat in Setup and uses it
// from mid-September. In the what-ifs with two providers, it also connects
// Second provider on 9 November. Second provider then joins the end of the
// order of each Linux label, and an Admin moves it before Boat in the order
// of ciproxy-large.

import { unbroken } from "./format.ts";
import type { ProviderKey } from "./providers.ts";

/** The overflow settings that hold for every provider. */
export const overflow = {
  /**
   * How long a job waits for a free slot on the own machines before it goes
   * to the next place of its label. It applies only when a job leaves the
   * own machines.
   */
  waitSeconds: 30,
  /** The UTC month whose spend counts against each limit: the budget month. */
  budgetMonth: "November",
  /** The first day of the next month, when each limit starts again. */
  nextMonth: unbroken("1 December"),
} as const;

/** The connected account of a provider. A screen never shows its key. */
export type ProviderAccount = {
  provider: ProviderKey;
  /** The name of the account at the provider. */
  account: string;
  /** The monthly spend limit, in US dollars. */
  spendLimit: number;
  /** The spend of November up to Wednesday 18 November, 14:32. */
  spentThisMonth: number;
};

/** The Boat account of mokly-ai. */
export const boat = {
  provider: "boat",
  account: "mokly",
  spendLimit: 200,
  spentThisMonth: 61.2,
  /** The day Boat stopped accepting the key, in a what-if. */
  keyRejectedOn: unbroken("12 November"),
} as const satisfies ProviderAccount & { keyRejectedOn: string };

/**
 * The Second provider account of mokly-ai in the what-ifs with two
 * providers. It took the overflow of ciproxy-large from 9 November, so its
 * spend is a part of the November spend that Boat has in the main story.
 */
export const secondProvider = {
  provider: "second",
  account: "mokly-ci",
  spendLimit: 100,
  spentThisMonth: 17.4,
} as const satisfies ProviderAccount;
