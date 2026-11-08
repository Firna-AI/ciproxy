// The workspace of the Stripe page that this tab opened. Stripe returns to one
// URL for all workspaces, so before a tab leaves for Stripe Checkout or the
// billing portal, it saves the exact workspace of that page in its own tab
// storage. A Stripe return without a workspace reads it from there. Normal
// navigation and other tabs never change it, unlike the workspace that the
// browser remembers for the landing.

import { idParam } from "../routing/pages.ts";
import type { KeyValueStorage } from "./storage.ts";

export const paymentContextKeys = {
  /** The workspace of the Checkout that this tab opened last. */
  checkout: "ciproxy.billing.checkout-workspace",
  /** The workspace of the billing portal that this tab opened last. */
  portal: "ciproxy.billing.portal-workspace",
} as const;

export type PaymentContextKind = keyof typeof paymentContextKeys;

export function savePaymentContext(
  tab: KeyValueStorage,
  kind: PaymentContextKind,
  workspaceId: string,
): void {
  tab.set(paymentContextKeys[kind], workspaceId);
}

/** The saved workspace of a Stripe page, or null when this tab saved none. */
export function paymentContext(
  tab: KeyValueStorage,
  kind: PaymentContextKind,
): string | null {
  return idParam(tab.get(paymentContextKeys[kind]) ?? undefined);
}

export function clearPaymentContext(
  tab: KeyValueStorage,
  kind: PaymentContextKind,
): void {
  tab.remove(paymentContextKeys[kind]);
}
