// The overflow providers of the mockups. Boat is the first provider.
// "Second provider" is the fixture name of a later provider, which the plan
// of the second real provider replaces with its brand. A provider is
// available when the deployment has its settings, and offered when the
// workspace connected it once. Both are available in the deployment of the
// story, so the provider choice of Setup and of Settings lists both. Product
// copy names the brand of a provider for its own things. Only Boat has
// wallets, a free trial, and start fees, so only the Boat branch of a form
// asks for a wallet.

/** A provider of ciproxy, as the API names it. */
export type ProviderKey = "boat" | "second";

/** The fixed facts of a provider that the screens show. */
export type Provider = {
  key: ProviderKey;
  /** The brand name, which product copy uses for the provider. */
  name: string;
  /** The line under the name in the provider choice. */
  summary: string;
  /** The start of a key, as the placeholder of the key field. */
  keyPlaceholder: string;
  /** Where a person creates a key, as the hint of the key field. */
  keyHint: string;
  /** The words of the error when the provider does not accept a key. */
  keyError: string;
  /**
   * A key can use several billing accounts, so a connection asks which one
   * after the key check: the wallets of Boat. A provider without a choice
   * connects in one step.
   */
  wallets: boolean;
};

export const providers: Readonly<Record<ProviderKey, Provider>> = {
  boat: {
    key: "boat",
    name: "Boat",
    summary: "Linux VMs in your Boat account, billed by the second.",
    keyPlaceholder: "boat_live_…",
    keyHint: "Create a key in the Boat console, under Settings.",
    keyError:
      "Boat did not accept this key. Copy the whole key from the Boat console.",
    wallets: true,
  },
  second: {
    key: "second",
    name: "Second provider",
    summary: "Linux VMs in your Second provider account.",
    keyPlaceholder: "Paste the key",
    keyHint: "Create a key in the console of Second provider.",
    keyError:
      "Second provider did not accept this key. Copy the whole key from its console.",
    wallets: false,
  },
};

/**
 * Every provider of ciproxy, in the order of the provider choice. The choice
 * shows a provider that is not available as unavailable right now.
 */
export const providerKeys: readonly ProviderKey[] = ["boat", "second"];

/** The brand name of a provider, such as "Boat". */
export function providerName(key: ProviderKey): string {
  return providers[key].name;
}

/** "Boat", "Boat or Second provider": the names of some providers. */
export function providerNames(
  keys: readonly ProviderKey[],
  join: "and" | "or" = "or",
): string {
  const names = keys.map(providerName);
  if (names.length < 2) return names.join("");
  return `${names.slice(0, -1).join(", ")} ${join} ${names.at(-1) ?? ""}`;
}
