// The wallets of a Boat key, as the wallet choice of a Boat connection shows
// them. Only Boat bills one of several wallets, so only the Boat branch of a
// connection form asks for one. The key check of the API returns them.

import type { BoatWallet } from "../api/schema.ts";

/** The identity of the personal wallet; each organization uses its Boat ID. */
export const personalWallet = "personal";

export const walletMissing = "Choose the wallet that Boat bills.";

/** One wallet as a radio card of the wallet choice. */
export type WalletOption = { value: string; title: string; body: string };

/** The wallets of a checked key, in the order of the API answer. */
export function walletOptions(wallets: readonly BoatWallet[]): WalletOption[] {
  return wallets.map((item) => ({
    value: item.wallet,
    title: item.name,
    body:
      item.wallet === personalWallet
        ? "Personal wallet"
        : "Boat organization wallet",
  }));
}
