// Why a provider cannot take a job now, in the words of the dashboard. The
// label list says "Waits: <reason>", an order row "<Reason>, so jobs skip
// it", and a Jobs notice "<Reason>." Each reason names its provider, except
// the workspace switch, which holds every provider.

import type { ProviderHold } from "../fixtures/placement.ts";
import { providerName, type ProviderKey } from "../fixtures/providers.ts";

/** "Boat reached its spend limit", or "overflow providers are off". */
export function holdText(provider: ProviderKey, hold: ProviderHold): string {
  const name = providerName(provider);
  switch (hold) {
    case "off":
      return "overflow providers are off";
    case "not-connected":
      return `${name} is not connected`;
    case "unavailable":
      return `${name} is unavailable right now`;
    case "limit":
      return `${name} reached its spend limit`;
    case "rejected":
      return `${name} rejected the key`;
    case "refusing":
      return `${name} refuses new sandboxes`;
    case "billing":
      return `${name} needs billing on your account`;
  }
}

/** The text with a capital first letter, to start a sentence. */
export function capitalized(text: string): string {
  return text.charAt(0).toUpperCase() + text.slice(1);
}
