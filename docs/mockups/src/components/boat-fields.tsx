// The two fields that connect a Boat account: its API key and the monthly
// spend limit. Every connection asks for both, in setup and in Settings, so a
// limit applies from the first job on Boat. A connection after Boat rejected
// the key, and a change of the key, ask for a new key, and the limit starts
// with the saved limit. A typed key shows masked.

import { typedKey } from "../fixtures/boat-form.ts";
import { TextField } from "./fields.tsx";

export function BoatFields({
  invalid = false,
  filled = false,
  limit,
  reconnect = false,
}: {
  /** Boat did not accept the key. */
  invalid?: boolean;
  /** The person typed a key, which shows masked. */
  filled?: boolean;
  /** The limit in the field, in US dollars: typed, or the saved limit. */
  limit?: number;
  /** A new key for a saved connection, so the limit is the saved limit. */
  reconnect?: boolean;
}) {
  const typed = invalid || filled;
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
