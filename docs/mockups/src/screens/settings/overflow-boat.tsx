// The Boat panel of Settings under Overflow: the connected account with
// Change key and Disconnect, and the form of each connection. Every
// connection asks for the key and the monthly spend limit, as setup does. The
// key check comes first: it lists the wallets of the key with the likely
// wallet picked, and Connect Boat then connects. A change of the key and a
// reconnect after Boat rejected the key start the limit with the saved
// limit. Every account keeps Disconnect, also after Boat rejected its key.
// Disconnect asks first, and says that running Boat jobs finish.

import { CloudOff } from "lucide-react";

import { LinkAction } from "../../components/actions.tsx";
import { BoatFields } from "../../components/boat-fields.tsx";
import { Dialog } from "../../components/dialog.tsx";
import { StateBlock } from "../../components/feedback.tsx";
import { Panel } from "../../components/page.tsx";
import { paths, type CataloguePath } from "../../components/paths.ts";
import { boat } from "../../fixtures/machines.ts";

const intro =
  "When your machines are busy, ciproxy starts jobs in your own Boat account. Boat bills you by the second.";

/** Check key before the key check, and Connect Boat after it. */
function Submit({ checked, to }: { checked: boolean; to: CataloguePath }) {
  return (
    <LinkAction to={to} tone="primary">
      {checked ? "Connect Boat" : "Check key"}
    </LinkAction>
  );
}

/** Boat is not connected yet, for example after Skip for now in setup. */
export function ConnectBoatPanel({ checked = false }: { checked?: boolean }) {
  return (
    <Panel title="Boat">
      <div className="stack settings-form">
        <p className="small">{intro}</p>
        <BoatFields
          wallets={checked}
          limit={checked ? boat.spendLimit : undefined}
        />
        <div className="row">
          <Submit
            checked={checked}
            to={checked ? paths.overflow : paths.overflowWalletConnect}
          />
        </div>
      </div>
    </Panel>
  );
}

/**
 * A new key after Boat rejected the saved one, with the saved limit. The
 * account keeps Disconnect, which opens the same dialog.
 */
export function ReconnectBoatPanel({ checked = false }: { checked?: boolean }) {
  return (
    <Panel title="Boat">
      <div className="stack settings-form">
        <BoatFields reconnect wallets={checked} limit={boat.spendLimit} />
        <div className="row-wrap">
          <Submit
            checked={checked}
            to={checked ? paths.overflow : paths.overflowWalletReconnect}
          />
          <LinkAction to={paths.overflowDisconnect} tone="danger">
            Disconnect
          </LinkAction>
        </div>
      </div>
    </Panel>
  );
}

/** Change key: a new connection, which asks for the limit again. */
export function ChangeKeyPanel({ checked = false }: { checked?: boolean }) {
  return (
    <Panel title="Boat">
      <div className="stack settings-form">
        <p className="small">
          New jobs start with the new key. Jobs that run on Boat now finish with
          the current key.
        </p>
        <BoatFields reconnect wallets={checked} limit={boat.spendLimit} />
        <div className="row-wrap">
          <Submit
            checked={checked}
            to={checked ? paths.overflow : paths.overflowWalletChangeKey}
          />
          <LinkAction to={paths.overflow} tone="ghost">
            Cancel
          </LinkAction>
        </div>
      </div>
    </Panel>
  );
}

/** The connected account, which shows its name and never the key. */
export function AccountPanel() {
  return (
    <Panel title="Boat">
      <div className="spread spread-stack">
        <div className="stack-tight">
          <p className="strong">Connected to the account {boat.account}</p>
          <p className="caption">
            ciproxy starts overflow jobs in this account. Boat bills it.
          </p>
        </div>
        <div className="row-wrap">
          <LinkAction to={paths.overflowChangeKey} size="sm">
            Change key
          </LinkAction>
          <LinkAction to={paths.overflowDisconnect} size="sm" tone="danger">
            Disconnect
          </LinkAction>
        </div>
      </div>
    </Panel>
  );
}

/** The deployment has no Boat service, so no connection can start. */
export function BoatUnavailablePanel() {
  return (
    <Panel title="Boat">
      <StateBlock
        icon={CloudOff}
        title="Boat is unavailable right now"
        action={
          <LinkAction to={paths.overflowUnavailable} tone="primary">
            Try again
          </LinkAction>
        }
      >
        Jobs run on your machines only.
      </StateBlock>
    </Panel>
  );
}

export function DisconnectDialog() {
  return (
    <Dialog
      title="Disconnect Boat?"
      footer={
        <>
          <LinkAction to={paths.overflow} tone="ghost">
            Cancel
          </LinkAction>
          <LinkAction to={paths.overflowNotConnected} tone="danger">
            Disconnect
          </LinkAction>
        </>
      }
    >
      <p className="small">
        ciproxy starts no new jobs on Boat, and jobs run on your machines only.
      </p>
      <p className="small">
        Jobs that run on Boat now finish. ciproxy then deletes the key.
      </p>
    </Dialog>
  );
}
