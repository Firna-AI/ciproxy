// Overflow settings: the workspace switch and the overflow wait, which hold
// for every provider, then a panel for each provider that is connected or
// available. A connected provider shows its account, its own monthly spend
// limit, and the spend of the month. While the deployment has no settings
// for a connected provider, its panel keeps the account and Disconnect. A
// provider that the workspace never connected shows Connect. With no
// connected provider, the page asks for a provider and its key and limit, as
// setup does. After a provider rejects the key, its panel asks for a new key
// and the limit again, and the limit starts with the saved limit. The labels
// set the order of the places.

import type { Viewport } from "@mokly/mokly";
import { DropdownSelector } from "@firna/ui/dropdown";
import { Switch } from "@firna/ui/switch";

import { noop, TextLink } from "../../components/actions.tsx";
import { Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { boat, overflow, secondProvider } from "../../fixtures/overflow.ts";
import { spendScenes } from "../../fixtures/provider-spend.ts";
import { waitChoices } from "../../fixtures/settings.ts";
import {
  AccountPanel,
  NotConnectedPanel,
  type SpendState,
} from "./overflow-account.tsx";
import {
  ChangeKeyPanel,
  ConnectPanel,
  ConnectSecondPanel,
  DisconnectDialog,
  NoProviderPanel,
  ReconnectPanel,
} from "./overflow-connect.tsx";
import { OverflowNotice } from "./overflow-notices.tsx";
import { SettingsFrame } from "./settings-frame.tsx";

export const overflowStates = [
  "settings",
  "limit-reached",
  "boat-rejected",
  "not-connected",
  "change-key",
  "disconnect",
  "unavailable",
  "unknown-costs",
  "spend-unavailable",
  "save-failed",
  "wallet-connect",
  "wallet-reconnect",
  "wallet-change-key",
  "two-providers",
  "connect-second",
  "second-limit-reached",
  "boat-unavailable",
] as const;
export type OverflowState = (typeof overflowStates)[number];

function waitLabel(seconds: number): string {
  if (seconds === 0) return "At once";
  if (seconds < 60) return `${seconds} seconds`;
  return seconds === 60 ? "1 minute" : `${seconds / 60} minutes`;
}

const waitOptions = waitChoices.map((seconds) => ({
  label: waitLabel(seconds),
  value: String(seconds),
}));

/** The workspace switch and the overflow wait, for every provider. */
function WaitPanel() {
  return (
    <Panel title="When jobs start on a provider">
      <div className="stack settings-form">
        <div className="toggle-row">
          <span className="stack-tight">
            <span className="strong">Use overflow providers</span>
            <span className="caption">
              Each label sets the order of its places, under{" "}
              <TextLink to={paths.settings}>Labels</TextLink>.
            </span>
          </span>
          <Switch
            value
            onValueChange={noop}
            accessibilityLabel="Use overflow providers"
          />
        </div>
        <DropdownSelector
          variant="field"
          label="Wait for a free slot on your machines"
          options={waitOptions}
          value={String(overflow.waitSeconds)}
          onValueChange={noop}
          hint="Then a job goes to the next place of its label."
        />
      </div>
    </Panel>
  );
}

/** The states in which Second provider is connected too. */
function withSecond(state: OverflowState): boolean {
  return (
    state === "two-providers" ||
    state === "second-limit-reached" ||
    state === "boat-unavailable"
  );
}

function boatSpendState(state: OverflowState): SpendState {
  if (state === "limit-reached") return "limit-reached";
  if (state === "spend-unavailable") return "unread";
  if (state === "save-failed") return "save-failed";
  return "steady";
}

/** The Boat panel of each state: its account, or the form in its place. */
function BoatPanel({ state }: { state: OverflowState }) {
  switch (state) {
    case "boat-rejected":
      return <ReconnectPanel />;
    case "wallet-reconnect":
      return <ReconnectPanel checked />;
    case "change-key":
      return <ChangeKeyPanel />;
    case "wallet-change-key":
      return <ChangeKeyPanel checked />;
    default:
      return (
        <AccountPanel
          account={boat}
          spend={
            state === "limit-reached"
              ? spendScenes.limitReached
              : withSecond(state)
                ? spendScenes.boatBesideSecond
                : spendScenes.steady
          }
          state={boatSpendState(state)}
          links={{
            changeKey: paths.overflowChangeKey,
            disconnect: paths.overflowDisconnect,
          }}
          available={state !== "boat-unavailable"}
        />
      );
  }
}

/** The Second provider panel: not connected, its connect form, or connected. */
function SecondPanel({ state }: { state: OverflowState }) {
  if (state === "connect-second") return <ConnectSecondPanel />;
  if (!withSecond(state)) {
    return (
      <NotConnectedPanel
        provider="second"
        connect={paths.overflowConnectSecond}
      />
    );
  }
  const reached = state === "second-limit-reached";
  return (
    <AccountPanel
      account={secondProvider}
      spend={reached ? spendScenes.secondLimitReached : spendScenes.second}
      state={reached ? "limit-reached" : "steady"}
    />
  );
}

export function OverflowScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: OverflowState;
}) {
  if (
    state === "not-connected" ||
    state === "wallet-connect" ||
    state === "unavailable"
  ) {
    return (
      <SettingsFrame viewport={viewport} active={paths.overflow}>
        {state === "unavailable" ? (
          <NoProviderPanel />
        ) : (
          <ConnectPanel checked={state === "wallet-connect"} />
        )}
      </SettingsFrame>
    );
  }
  return (
    <SettingsFrame
      viewport={viewport}
      active={paths.overflow}
      overlay={state === "disconnect" ? <DisconnectDialog /> : undefined}
    >
      <OverflowNotice state={state} />
      <WaitPanel />
      <BoatPanel state={state} />
      <SecondPanel state={state} />
    </SettingsFrame>
  );
}
