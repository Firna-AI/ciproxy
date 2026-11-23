// The panel of one provider on Settings under Overflow. A connected provider
// shows the name of its account, never the key, with Change key and
// Disconnect, and its own monthly spend limit with the spend of the month:
// the costs that the provider confirmed and the costs that it has not
// confirmed yet, apart. When ciproxy cannot read the spend, the panel says
// so, and the limit still applies. A failed save keeps the typed limit. An
// available provider that the workspace never connected shows what it is
// and Connect: it then goes last in the order of each Linux label, as the
// answer to item 6 of the provider plan says.

import { CircleAlert, Plus } from "lucide-react";

import { Action, LinkAction } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { TextField } from "../../components/fields.tsx";
import { Panel } from "../../components/page.tsx";
import type { CataloguePath } from "../../components/paths.ts";
import { SpendFigures } from "../../components/provider-spend.tsx";
import type { ProviderAccount } from "../../fixtures/overflow.ts";
import { unsavedLimit } from "../../fixtures/provider-form.ts";
import type { ProviderSpend } from "../../fixtures/provider-spend.ts";
import {
  providerName,
  providers,
  type ProviderKey,
} from "../../fixtures/providers.ts";

/** The spend panel: steady, at the limit, not read, or not saved. */
export type SpendState = "steady" | "limit-reached" | "unread" | "save-failed";

/** The screens of Change key and Disconnect, when the catalogue has them. */
export type AccountLinks = {
  changeKey: CataloguePath;
  disconnect: CataloguePath;
};

function SpendUnavailable({ name }: { name: string }) {
  return (
    <div className="spend-missing">
      <CircleAlert size={16} className="icon-muted" />
      <div className="stack-tight">
        <p className="strong">Unable to read the {name} spend</p>
        <p className="caption">The limit still applies.</p>
      </div>
      <Action size="sm">Try again</Action>
    </div>
  );
}

/** Change key, while the provider is available, and Disconnect. */
function AccountActions({
  links,
  available,
}: {
  links?: AccountLinks;
  available: boolean;
}) {
  const changeKey = links ? (
    <LinkAction to={links.changeKey} size="sm">
      Change key
    </LinkAction>
  ) : (
    <Action size="sm">Change key</Action>
  );
  return (
    <div className="row-wrap">
      {available ? changeKey : null}
      {links ? (
        <LinkAction to={links.disconnect} size="sm" tone="danger">
          Disconnect
        </LinkAction>
      ) : (
        <Action size="sm" tone="danger">
          Disconnect
        </Action>
      )}
    </div>
  );
}

/**
 * A connected provider: its account, its actions, and its spend limit. While
 * the deployment has no settings for the provider, the account stays with
 * Disconnect, and no new connection can start, so Change key leaves.
 */
export function AccountPanel({
  account,
  spend,
  state = "steady",
  links,
  available = true,
}: {
  account: ProviderAccount;
  spend: ProviderSpend;
  state?: SpendState;
  links?: AccountLinks;
  /** The deployment has the settings of the provider. */
  available?: boolean;
}) {
  const name = providerName(account.provider);
  const failed = state === "save-failed";
  return (
    <Panel title={name}>
      <div className="stack">
        <div className="spread spread-stack">
          <div className="stack-tight">
            <p className="strong">Connected to the account {account.account}</p>
            <p className="caption">
              {available
                ? `ciproxy starts jobs in this account. ${name} bills it.`
                : `${name} is unavailable right now, so jobs skip it. ciproxy keeps the account and its limit.`}
            </p>
          </div>
          <AccountActions links={links} available={available} />
        </div>
        <div className="provider-limit">
          <div className="stack settings-form">
            <TextField
              label="Monthly spend limit, US dollars"
              value={String(failed ? unsavedLimit : account.spendLimit)}
            />
            {state === "unread" ? (
              <SpendUnavailable name={name} />
            ) : (
              <SpendFigures
                provider={account.provider}
                spend={spend}
                limit={account.spendLimit}
                tone={state === "limit-reached" ? "warning" : "normal"}
              />
            )}
            {failed ? (
              <Notice tone="danger" title="Unable to save the limit">
                Try Save limit again.
              </Notice>
            ) : null}
            <div className="row">
              <Action tone="primary">Save limit</Action>
            </div>
          </div>
        </div>
      </div>
    </Panel>
  );
}

/** An available provider that the workspace never connected. */
export function NotConnectedPanel({
  provider,
  connect,
}: {
  provider: ProviderKey;
  connect: CataloguePath;
}) {
  const facts = providers[provider];
  return (
    <Panel title={facts.name}>
      <div className="spread spread-stack">
        <div className="stack-tight">
          <p className="small">{facts.summary}</p>
          <p className="caption">
            When you connect it, it goes last in the order of each Linux label.
          </p>
        </div>
        <LinkAction to={connect} size="sm" icon={Plus}>
          Connect {facts.name}
        </LinkAction>
      </div>
    </Panel>
  );
}
