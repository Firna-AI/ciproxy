// The Boat wallets that Cal Moore's key can use, as the key check lists them:
// his personal wallet and the wallet of each Boat organization that he
// belongs to. Boat bills each organization to its own wallet. The key check
// names the mokly organization as the likely wallet, so the form preselects
// it, and Cal can pick another one before he connects.

export type BoatWalletKind = "personal" | "organization";

export type BoatWallet = {
  name: string;
  kind: BoatWalletKind;
  /** The wallet that the key check names as likely, which starts picked. */
  likely: boolean;
};

export const boatWallets: readonly BoatWallet[] = [
  { name: "Cal Moore", kind: "personal", likely: false },
  { name: "mokly", kind: "organization", likely: true },
  { name: "driftwood", kind: "organization", likely: false },
];
