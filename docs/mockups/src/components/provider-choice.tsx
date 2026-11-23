// The provider choice of a connection, in Setup and in Settings under
// Overflow: a card for each provider of ciproxy, with the picked one
// checked. The form of the picked provider shows below it, and its submit
// button: Check key before the Boat key check and Connect Boat after it, or
// Connect for a provider with no choice of billing account. A provider that
// is not available, because the deployment has no settings for it, shows as
// unavailable right now and cannot be picked.

import { RadioCard, RadioCardGroup } from "@firna/ui/radio";

import {
  providerKeys,
  providers,
  type ProviderKey,
} from "../fixtures/providers.ts";
import { LinkAction, noop } from "./actions.tsx";
import type { CataloguePath } from "./paths.ts";

/** The first sentence of a connection, in Setup and in Settings. */
export const connectIntro =
  "When your machines are busy, ciproxy starts jobs in your own account at an overflow provider, which bills you.";

/** Check key before the Boat key check, and Connect after it. */
export function SubmitKey({
  provider,
  checked,
  to,
}: {
  provider: ProviderKey;
  checked: boolean;
  to: CataloguePath;
}) {
  const facts = providers[provider];
  return (
    <LinkAction to={to} tone="primary">
      {facts.wallets && !checked ? "Check key" : `Connect ${facts.name}`}
    </LinkAction>
  );
}

export function ProviderChoice({
  picked,
  unavailable = [],
}: {
  /** The provider whose form shows, when one can be picked. */
  picked?: ProviderKey;
  /** The providers that the deployment cannot start now. */
  unavailable?: readonly ProviderKey[];
}) {
  return (
    <RadioCardGroup label="Overflow provider">
      {providerKeys.map((key) => {
        const off = unavailable.includes(key);
        return (
          <RadioCard
            key={key}
            title={providers[key].name}
            body={off ? "Unavailable right now" : providers[key].summary}
            checked={!off && key === picked}
            disabled={off}
            onPress={noop}
          />
        );
      })}
    </RadioCardGroup>
  );
}
