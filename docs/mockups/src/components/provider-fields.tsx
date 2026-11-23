// The two fields that connect a provider account: its API key and the
// monthly spend limit of that provider. Every connection asks for both, in
// setup and in Settings, so a limit applies from the first job on the
// provider. A connection after the provider rejected the key, and a change
// of the key, ask for a new key, and the limit starts with the saved limit.
// A typed key shows masked. In the Boat branch, the wallet choice shows
// between the key and the limit after the key check.

import { typedKeys } from "../fixtures/provider-form.ts";
import { providers, type ProviderKey } from "../fixtures/providers.ts";
import { BoatWallets } from "./boat-wallets.tsx";
import { TextField } from "./fields.tsx";

export function ProviderFields({
  provider,
  invalid = false,
  wallets = false,
  typed = false,
  limit,
  reconnect = false,
}: {
  provider: ProviderKey;
  /** The provider did not accept the key. */
  invalid?: boolean;
  /** The Boat key check passed, so the wallets of the key show. */
  wallets?: boolean;
  /** The person typed the key, as before a connection that did not finish. */
  typed?: boolean;
  /** The limit in the field, in US dollars: typed, or the saved limit. */
  limit?: number;
  /** A new key for a saved connection, so the limit is the saved limit. */
  reconnect?: boolean;
}) {
  const facts = providers[provider];
  const keyTyped = invalid || wallets || typed;
  const atLimit = `At the limit, ciproxy starts no new jobs on ${facts.name} until the next month.`;
  return (
    <>
      <TextField
        label={`${reconnect ? "New " : ""}${facts.name} API key`}
        placeholder={facts.keyPlaceholder}
        value={keyTyped ? typedKeys[provider] : undefined}
        hint={invalid ? undefined : facts.keyHint}
        error={invalid ? facts.keyError : undefined}
        secret={keyTyped}
      />
      {wallets && facts.wallets ? <BoatWallets /> : null}
      <TextField
        label={`Monthly spend limit on ${facts.name}, US dollars`}
        placeholder="For example, 200"
        value={limit === undefined ? undefined : String(limit)}
        hint={reconnect ? `Your saved limit. ${atLimit}` : atLimit}
      />
    </>
  );
}
