// The list prices that the cost claim of the landing page compares: one new
// Hetzner AX42, and the prices per minute of Blacksmith and GitHub-hosted
// runners with 2 vCPU. Each price keeps the page that shows it and the day it
// was checked. Check them again each month after the launch, and before it.

/** A list price as the vendor's page showed it, in US dollars without tax. */
export type ListPrice = {
  amount: number;
  /** What the price buys, in the words of the vendor. */
  item: string;
  /** The page that shows the price. */
  source: string;
  /** The day the page showed this price. */
  checked: string;
};

const checked = "4 November 2026";
const hetznerPage = "https://www.hetzner.com/dedicated-rootserver/ax42/";

/**
 * The list prices. The Hetzner page shows €99.00 a month by default, and
 * $119.00 with its currency set to US dollars. Both include the dedicated
 * IPv4 address of the server.
 */
export const listPrices = {
  machine: {
    amount: 119,
    item: "AX42, per month, with its IPv4 address",
    source: hetznerPage,
    checked,
  },
  machineSetup: {
    amount: 59,
    item: "AX42, one-time setup fee",
    source: hetznerPage,
    checked,
  },
  blacksmith: {
    amount: 0.004,
    item: "Ubuntu x64, 2 vCPU and 8 GB, per minute",
    source: "https://www.blacksmith.sh/pricing",
    checked,
  },
  github: {
    amount: 0.006,
    item: "Linux 2-core (x64), per minute",
    source:
      "https://docs.github.com/en/billing/reference/actions-runner-pricing",
    checked,
  },
} as const satisfies Readonly<Record<string, ListPrice>>;
