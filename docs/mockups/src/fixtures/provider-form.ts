// What Cal Moore typed in the provider forms of the mockups: a key for each
// provider that shows masked, and a new limit whose save did not finish.
// None of them is a real key or a saved value.

import type { ProviderKey } from "./providers.ts";

/** A typed key of each provider. A field shows it masked, never in plain text. */
export const typedKeys: Readonly<Record<ProviderKey, string>> = {
  boat: "boat_live_3kQ9xV2",
  second: "sp_7Hc2Lq9Wd",
};

/** A typed monthly limit, in US dollars, whose save did not finish. */
export const unsavedLimit = 300;
