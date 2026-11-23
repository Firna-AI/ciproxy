// The forms of a connection on Settings under Overflow. With no connected
// provider, the page asks for a provider first, as setup does, and shows the
// form of the picked one: the key and the monthly spend limit. The Boat key
// check comes first: it lists the wallets of the key with the likely wallet
// picked, and Connect Boat then connects. A provider with no choice of
// billing account connects in one step. A change of the key and a reconnect
// after the provider rejected the key start the limit with the saved limit.
// Every account keeps Disconnect, also after the provider rejected its key.
// Disconnect asks first: running jobs finish, and the labels keep the
// provider in their order and skip it.

import { CloudOff } from "lucide-react";

import { LinkAction } from "../../components/actions.tsx";
import { Dialog } from "../../components/dialog.tsx";
import { StateBlock } from "../../components/feedback.tsx";
import { Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import {
  connectIntro,
  ProviderChoice,
  SubmitKey,
} from "../../components/provider-choice.tsx";
import { ProviderFields } from "../../components/provider-fields.tsx";
import { boat } from "../../fixtures/overflow.ts";
import { providers } from "../../fixtures/providers.ts";

/** No provider is connected: the choice, then the form of the picked one. */
export function ConnectPanel({ checked = false }: { checked?: boolean }) {
  return (
    <Panel title="Connect an overflow provider">
      <div className="stack settings-form">
        <p className="small">{connectIntro}</p>
        <ProviderChoice picked="boat" />
        <ProviderFields
          provider="boat"
          wallets={checked}
          limit={checked ? boat.spendLimit : undefined}
        />
        <div className="row">
          <SubmitKey
            provider="boat"
            checked={checked}
            to={checked ? paths.overflow : paths.overflowWalletConnect}
          />
        </div>
      </div>
    </Panel>
  );
}

/** The deployment can start no provider now, so no connection can start. */
export function NoProviderPanel() {
  return (
    <Panel title="Connect an overflow provider">
      <StateBlock
        icon={CloudOff}
        title="No overflow provider is available right now"
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

/**
 * A new Boat key after Boat rejected the saved one, with the saved limit.
 * The account keeps Disconnect, which opens the same dialog.
 */
export function ReconnectPanel({ checked = false }: { checked?: boolean }) {
  return (
    <Panel title="Boat">
      <div className="stack settings-form">
        <ProviderFields
          provider="boat"
          reconnect
          wallets={checked}
          limit={boat.spendLimit}
        />
        <div className="row-wrap">
          <SubmitKey
            provider="boat"
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

/** Change key of Boat: a new connection, which asks for the limit again. */
export function ChangeKeyPanel({ checked = false }: { checked?: boolean }) {
  return (
    <Panel title="Boat">
      <div className="stack settings-form">
        <p className="small">
          New jobs start with the new key. Jobs that run on Boat now finish with
          the current key.
        </p>
        <ProviderFields
          provider="boat"
          reconnect
          wallets={checked}
          limit={boat.spendLimit}
        />
        <div className="row-wrap">
          <SubmitKey
            provider="boat"
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

/** Connect of an available provider, beside a connected one. */
export function ConnectSecondPanel() {
  return (
    <Panel title={providers.second.name}>
      <div className="stack settings-form">
        <p className="small">
          It goes last in the order of each Linux label. You can move it or
          remove it in each label.
        </p>
        <ProviderFields provider="second" />
        <div className="row-wrap">
          <SubmitKey
            provider="second"
            checked={false}
            to={paths.overflowTwoProviders}
          />
          <LinkAction to={paths.overflow} tone="ghost">
            Cancel
          </LinkAction>
        </div>
      </div>
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
        ciproxy starts no new jobs on Boat. Your labels keep Boat in their order
        and skip it until you connect Boat again.
      </p>
      <p className="small">
        Jobs that run on Boat now finish. ciproxy then deletes the key.
      </p>
    </Dialog>
  );
}
