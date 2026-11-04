// The two fields that connect a Boat account: its API key and the monthly
// spend limit. Every connection asks for both, in setup and in Settings, so a
// limit applies from the first job on Boat.

import { TextField } from "./fields.tsx";

export function BoatFields({
  invalid = false,
  limit,
}: {
  /** Boat did not accept the key. */
  invalid?: boolean;
  /** The limit that the customer typed, in US dollars. */
  limit?: number;
}) {
  return (
    <>
      <TextField
        label="Boat API key"
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
        hint="At the limit, jobs run on your machines only until the next month."
      />
    </>
  );
}
