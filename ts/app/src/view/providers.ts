// The copy of each overflow provider that the screens show, and the words of
// a reason why a provider cannot take a job now. The descriptor of a provider
// on the wire holds facts only, such as whether a connection has a choice of
// billing account. The app keeps the copy of each provider: its brand name,
// the line of the provider choice, and the placeholder, hint, and error of
// its key field. Jobs, History, and Savings name a provider only by its
// value, so every screen takes the brand name from here. A new provider adds
// its copy here, and the record type makes the compiler ask for it.

import type { OverflowProvider, ProviderHold } from "../api/schema.ts";

export type ProviderCopy = {
  /** The brand name, which product copy uses for the provider. */
  name: string;
  /** The line under the name in the provider choice. */
  summary: string;
  /** The start of a key, as the placeholder of the key field. */
  keyPlaceholder: string;
  /** Where a person creates a key, as the hint of the key field. */
  keyHint: string;
  /** The error of the key field when the provider does not accept a key. */
  keyError: string;
};

export const providerCopy: Readonly<Record<OverflowProvider, ProviderCopy>> = {
  boat: {
    name: "Boat",
    summary: "Linux VMs in your Boat account, billed by the second.",
    keyPlaceholder: "boat_live_…",
    keyHint: "Create a key in the Boat console, under Settings.",
    keyError:
      "Boat did not accept this key. Copy the whole key from the Boat console.",
  },
};

function isProvider(value: string): value is OverflowProvider {
  return Object.hasOwn(providerCopy, value);
}

/** Every provider of ciproxy, in the order of the copy above. */
export const allProviders: readonly OverflowProvider[] =
  Object.keys(providerCopy).filter(isProvider);

/** The provider that a text names, such as the query of an address. */
export function providerOf(value: string | null): OverflowProvider | null {
  return value !== null && isProvider(value) ? value : null;
}

/** The brand name of a provider, such as "Boat". */
export function providerName(provider: OverflowProvider): string {
  return providerCopy[provider].name;
}

/** "Boat", "Boat or Second provider", "A, B, and C": some providers. */
export function providerNames(
  providers: readonly OverflowProvider[],
  join: "and" | "or" = "or",
): string {
  const names = providers.map(providerName);
  const last = names.pop();
  if (last === undefined) return "";
  if (names.length === 0) return last;
  const comma = names.length > 1 ? "," : "";
  return `${names.join(", ")}${comma} ${join} ${last}`;
}

/**
 * Why a provider cannot take a job now, as one clause: "Boat reached its
 * spend limit", or "overflow providers are off", which holds every provider.
 * The label list says "Waits: <clause>", an order row "<Clause>, so jobs
 * skip it", and a Jobs notice "<Clause>."
 */
export function holdText(
  provider: OverflowProvider,
  hold: ProviderHold,
): string {
  const name = providerName(provider);
  switch (hold) {
    case "off":
      return "overflow providers are off";
    case "not_connected":
      return `${name} is not connected`;
    case "unavailable":
      return `${name} is unavailable right now`;
    case "key_rejected":
      return `${name} rejected the key`;
    case "spend_limit":
      return `${name} reached its spend limit`;
    case "costs_unknown":
      return `${name} costs are not known yet`;
    case "retry_after":
      return `${name} refuses new sandboxes`;
    case "billing_required":
      return `${name} needs billing on your account`;
    case "size_refused":
      return `${name} refused this size`;
  }
}

/** The text with a capital first letter, to start a sentence. */
export function capitalized(text: string): string {
  return text.charAt(0).toUpperCase() + text.slice(1);
}
