// The open payment of a workspace that this tab can continue: the exact
// Stripe URL that the server returned for it, a Checkout page or a bank
// confirmation page. The tab keeps it in its own storage, one record for
// each workspace, so a reload or a return from Stripe can continue the same
// payment without a new payment request. Each server outcome replaces the
// record, and a read that shows no pending payment removes it, so an older
// payment is never offered again.

import type { PaymentOutcome } from "../api/billing-types.ts";
import type { KeyValueStorage } from "./storage.ts";

export type OpenPayment = {
  /** The exact Stripe URL that the server returned. */
  url: string;
  /** A Checkout page, or the page where the bank confirms a charge. */
  kind: "checkout" | "bank";
};

const prefix = "ciproxy.billing.open-payment.";

/** The tab-storage key of the open payment of one workspace. */
export function openPaymentKey(workspaceId: string): string {
  return `${prefix}${workspaceId}`;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

/** The saved open payment of a workspace, or null for none or a bad record. */
export function openPayment(
  tab: KeyValueStorage,
  workspaceId: string,
): OpenPayment | null {
  const text = tab.get(openPaymentKey(workspaceId));
  if (text === null) return null;
  let value: unknown;
  try {
    value = JSON.parse(text);
  } catch {
    return null;
  }
  if (!isRecord(value) || typeof value.url !== "string" || value.url === "") {
    return null;
  }
  return value.kind === "checkout" || value.kind === "bank"
    ? { url: value.url, kind: value.kind }
    : null;
}

export function clearOpenPayment(
  tab: KeyValueStorage,
  workspaceId: string,
): void {
  tab.remove(openPaymentKey(workspaceId));
}

/**
 * Keep the URL of an outcome that the person can continue: an open Checkout,
 * or a bank that asks to confirm. Every other outcome ends what was saved.
 */
export function recordOutcome(
  tab: KeyValueStorage,
  workspaceId: string,
  outcome: PaymentOutcome,
): void {
  const saved: OpenPayment | null =
    outcome.state === "open"
      ? { url: outcome.url, kind: "checkout" }
      : outcome.state === "payment_required" && outcome.payment_url
        ? { url: outcome.payment_url, kind: "bank" }
        : null;
  if (saved === null) clearOpenPayment(tab, workspaceId);
  else tab.set(openPaymentKey(workspaceId), JSON.stringify(saved));
}
