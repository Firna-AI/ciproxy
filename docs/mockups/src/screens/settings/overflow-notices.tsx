// The notices of Settings under Overflow. Each names the provider that it is
// about, because each provider has its own key and its own spend limit, and
// says when ciproxy starts jobs on that provider again. The other providers
// of a label order go on as before.

import { Notice } from "../../components/feedback.tsx";
import { formatMoney } from "../../fixtures/format.ts";
import { boat, overflow, secondProvider } from "../../fixtures/overflow.ts";
import type { ProviderAccount } from "../../fixtures/overflow.ts";
import { providerName } from "../../fixtures/providers.ts";
import type { OverflowState } from "./overflow.tsx";

function LimitNotice({ account }: { account: ProviderAccount }) {
  const name = providerName(account.provider);
  return (
    <Notice
      tone="warning"
      title={`${name} spend reached the ${formatMoney(account.spendLimit)} limit for ${overflow.budgetMonth}`}
    >
      ciproxy starts no new jobs on {name} until {overflow.nextMonth}, or until
      you raise the limit.
    </Notice>
  );
}

export function OverflowNotice({ state }: { state: OverflowState }) {
  switch (state) {
    case "limit-reached":
      return <LimitNotice account={boat} />;
    case "second-limit-reached":
      return <LimitNotice account={secondProvider} />;
    case "boat-rejected":
    case "wallet-reconnect":
      return (
        <Notice
          tone="danger"
          title={`Boat rejected the API key on ${boat.keyRejectedOn}`}
        >
          ciproxy starts no new jobs on Boat until you connect a new key.
        </Notice>
      );
    case "boat-unavailable":
      return (
        <Notice tone="warning" title="Boat is unavailable right now">
          ciproxy starts no new jobs on Boat until it is available again. Jobs
          skip Boat and go on to the next place of their label.
        </Notice>
      );
    case "unknown-costs":
      return (
        <Notice tone="warning" title="Boat costs are not known yet">
          ciproxy starts no new jobs on Boat until Boat reports the cost of each
          start.
        </Notice>
      );
    default:
      return null;
  }
}
