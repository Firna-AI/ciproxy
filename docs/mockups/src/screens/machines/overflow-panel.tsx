// The overflow of the workspace on the Machines page: a row for each
// connected provider, with its account and the spend of the month against
// its own limit, the confirmed and the unconfirmed costs apart, and the
// overflow wait under the rows. With no connected provider, the panel says
// that jobs run on the machines only. Only an Admin gets the link to the
// overflow settings and Connect a provider. When a later read of the panel
// fails, the panel keeps the figures of its last good read, and a warning
// above them names the failure, with Try again.

import { Cloud, CloudOff } from "lucide-react";

import { LinkAction, TextLink } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { SpendFigures } from "../../components/provider-spend.tsx";
import { unbroken } from "../../fixtures/format.ts";
import {
  boat,
  overflow,
  secondProvider,
  type ProviderAccount,
} from "../../fixtures/overflow.ts";
import {
  spendScenes,
  type ProviderSpend,
} from "../../fixtures/provider-spend.ts";
import { providerName } from "../../fixtures/providers.ts";

/** A connected provider and its spend of the month. */
export type AccountSpend = { account: ProviderAccount; spend: ProviderSpend };

/** The connected providers of each moment of the Machines page. */
export const overflowScenes = {
  boatOnly: [{ account: boat, spend: spendScenes.steady }],
  twoProviders: [
    { account: boat, spend: spendScenes.boatBesideSecond },
    { account: secondProvider, spend: spendScenes.second },
  ],
  none: [],
} as const satisfies Record<string, readonly AccountSpend[]>;

function ProviderRow({ account, spend }: AccountSpend) {
  return (
    <div className="overflow-row">
      <span className="icon-tile">
        <Cloud size={16} />
      </span>
      <div className="stack-tight overflow-text">
        <p className="strong">{providerName(account.provider)}</p>
        <p className="caption">Connected to the account {account.account}</p>
      </div>
      <div className="overflow-meter">
        <SpendFigures
          provider={account.provider}
          spend={spend}
          limit={account.spendLimit}
          compact
        />
      </div>
    </div>
  );
}

function NoProviderRow({ settings }: { settings: boolean }) {
  return (
    <div className="overflow-row">
      <span className="icon-tile">
        <CloudOff size={16} />
      </span>
      <div className="stack-tight overflow-text">
        <p className="strong">Jobs run on your machines only</p>
        <p className="caption">
          Connect an overflow provider to run jobs when your machines are busy.
        </p>
      </div>
      {settings ? (
        <LinkAction to={paths.overflowNotConnected} size="sm">
          Connect a provider
        </LinkAction>
      ) : null}
    </div>
  );
}

export function OverflowPanel({
  settings,
  refreshFailed = false,
  accounts = overflowScenes.boatOnly,
}: {
  settings: boolean;
  /** A later read failed, so the panel shows the figures of the last one. */
  refreshFailed?: boolean;
  /** The connected providers, in the order of their first connection. */
  accounts?: readonly AccountSpend[];
}) {
  return (
    <Panel
      title="Overflow"
      action={
        settings ? (
          <TextLink to={paths.overflow}>Overflow settings</TextLink>
        ) : undefined
      }
    >
      <div className="overflow-body">
        {refreshFailed ? (
          <Notice
            tone="warning"
            title="Unable to refresh overflow"
            actions={
              <LinkAction to={paths.machines} size="sm">
                Try again
              </LinkAction>
            }
          >
            Overflow shows the last state that it read.
          </Notice>
        ) : null}
        {accounts.length === 0 ? (
          <NoProviderRow settings={settings} />
        ) : (
          <>
            {accounts.map((entry) => (
              <ProviderRow key={entry.account.provider} {...entry} />
            ))}
            <p className="caption">
              Jobs wait {unbroken(`${overflow.waitSeconds} seconds`)} for a free
              slot on your machines, then go to the next place of their label.
            </p>
          </>
        )}
      </div>
    </Panel>
  );
}
