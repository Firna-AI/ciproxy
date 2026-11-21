// When a billing page of one workspace waits, and how often it reads again.
// Plan and usage and Plans are page stores. While Stripe confirms a payment
// or a billing operation runs, they read again on this interval, so the
// server's outcome replaces the wait by itself.

import type { PlanStatus } from "../api/billing-types.ts";

/** Whether a billing status waits for Stripe or for a running operation. */
export function statusWaits(status: PlanStatus): boolean {
  return status.payment_pending || status.operation_in_progress;
}

export const billingPollInterval = 5_000;
