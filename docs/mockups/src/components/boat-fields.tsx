// The two fields that connect a Boat account: its API key and the monthly
// spend limit. Every connection asks for both, in setup and in Settings, so a
// limit applies from the first job on Boat. A connection after Boat rejected
// the key asks for a new key, and the limit starts with the saved limit.

import { TextField } from "./fields.tsx";

export function BoatFields({
  invalid = false,
  limit,
  reconnect = false,
}: {
  /** Boat did not accept the key. */
  invalid?: boolean;
  /** The limit in the field, in US dollars: typed, or the saved limit. */
  limit?: number;
  /** Boat rejected the saved key, so the person connects a new key. */
  reconnect?: boolean;
}) {
  return (
    <>
      <TextField
        label={reconnect ? "New Boat API key" : "Boat API key"}
        placeholder="boat_live_…"
        value={invalid ? "boat_live_3kQ9xV2" : undefined}
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
        secret={invalid}
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
