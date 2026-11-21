// The wallet choice of a Boat connection, after the key check: the personal
// wallet and each Boat organization wallet that the key can use, with the
// likely wallet preselected. Setup and Settings under Overflow show it for a
// first connection, a reconnect, and Change key. A change of the key clears
// the choice, and the key check runs again.

import { RadioCard, RadioCardGroup } from "@firna/ui/radio";

import { boatWallets, type BoatWalletKind } from "../fixtures/boat-wallets.ts";
import { noop } from "./actions.tsx";

const kindText: Record<BoatWalletKind, string> = {
  personal: "Personal wallet",
  organization: "Boat organization wallet",
};

export function BoatWallets() {
  return (
    <div className="stack-tight">
      <RadioCardGroup label="Wallet that Boat bills">
        {boatWallets.map((wallet) => (
          <RadioCard
            key={wallet.name}
            title={wallet.name}
            body={kindText[wallet.kind]}
            checked={wallet.likely}
            onPress={noop}
          />
        ))}
      </RadioCardGroup>
      <p className="caption">
        Your key can use these wallets. Each Boat organization has its own
        wallet.
      </p>
    </div>
  );
}
