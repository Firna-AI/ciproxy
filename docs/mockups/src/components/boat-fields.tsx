// The two fields that connect a Boat account: its API key and the monthly
// spend limit. Every connection asks for both, in setup and in Settings, so a
// limit applies from the first job on Boat. A connection after Boat rejected
// the key, and a change of the key, ask for a new key, and the limit starts
// with the saved limit. A typed key shows masked. After the key check, the
// wallet choice shows between the key and the limit.

import { typedKey } from "../fixtures/boat-form.ts";
import { BoatWallets } from "./boat-wallets.tsx";
import { TextField } from "./fields.tsx";

export function BoatFields({
  invalid = false,
  wallets = false,
  limit,
  reconnect = false,
}: {
  /** Boat did not accept the key. */
  invalid?: boolean;
  /** The key check passed, so the wallets of the key show. */
  wallets?: boolean;
  /** The limit in the field, in US dollars: typed, or the saved limit. */
  limit?: number;
  /** A new key for a saved connection, so the limit is the saved limit. */
  reconnect?: boolean;
}) {
  const typed = invalid || wallets;
  return (
    <>
      <TextField
        label={reconnect ? "New Boat API key" : "Boat API key"}
        placeholder="boat_live_…"
        value={typed ? typedKey : undefined}
        hint={
          invalid
            ? undefined
            : "Create a key in the Boat console, under Settings."
        }
        error={
          invalid
            ? "Boat did not accept this key. Copy the whole key from the Boat console."
            : undefined
        }
        secret={typed}
      />
      {wallets ? <BoatWallets /> : null}
      <TextField
        label="Monthly spend limit, US dollars"
        placeholder="For example, 200"
        value={limit === undefined ? undefined : String(limit)}
        hint={
          reconnect
            ? "Your saved limit. At the limit, jobs run on your machines only until the next month."
            : "At the limit, jobs run on your machines only until the next month."
        }
      />
    </>
  );
}
